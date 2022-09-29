import "@polkadot/api-augment";

import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { blake2AsHex } from "@polkadot/util-crypto";
import chalk from "chalk";
import { ethers } from "ethers";
import { sha256 } from "ethers/lib/utils";
import fs from "fs";
import { HttpProvider } from "web3-core";

import { DEBUG_MODE } from "./constants";
import { cancelReferendaWithCouncil, executeProposalWithCouncil } from "./governance";
import {
  getRuntimeWasm,
  NodePorts,
  ParachainPorts,
  ParaTestOptions,
  startParachainNodes,
  stopParachainNodes,
} from "./para-node";
import { EnhancedWeb3, provideEthersApi, providePolkadotApi, provideWeb3Api } from "./providers";

const debug = require("debug")("test:setup");

const PORT_PREFIX = (process.env.PORT_PREFIX && parseInt(process.env.PORT_PREFIX)) || 19;

export interface ParaTestContext {
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;
  createPolkadotApiParachain: (parachainNumber: number) => Promise<ApiPromise>;
  createPolkadotApiParachains: () => Promise<ApiPromise>;
  createPolkadotApiRelaychains: () => Promise<ApiPromise>;
  waitBlocks: (count: number) => Promise<number>; // return current block when the promise resolves
  upgradeRuntime: (
    from: KeyringPair,
    runtimeName: "moonbase" | "moonriver" | "moonbeam",
    runtimeVersion: string,
    options?: { waitMigration?: boolean; useGovernance?: boolean }
  ) => Promise<number>;
  blockNumber: number;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApiParaone: ApiPromise;
}

export interface ParachainApis {
  parachainId: number;
  apis: ApiPromise[];
}

export interface InternalParaTestContext extends ParaTestContext {
  _polkadotApiParachains: ParachainApis[];
  _polkadotApiRelaychains: ApiPromise[];
  _web3Providers: HttpProvider[];
}

export function describeParachain(
  title: string,
  options: ParaTestOptions,
  cb: (context: InternalParaTestContext) => void
) {
  describe(title, function () {
    // Set timeout to 5000 for all tests.
    this.timeout("spec" in options.parachain ? 3600000 : 300000);

    // The context is initialized empty to allow passing a reference
    // and to be filled once the node information is retrieved
    let context: InternalParaTestContext = {} as InternalParaTestContext;

    // Making sure the Moonbeam node has started
    before("Starting Moonbeam Test Node", async function () {
      try {
        const init = !DEBUG_MODE
          ? await startParachainNodes(options)
          : {
              paraPorts: [
                {
                  parachainId: 1000,
                  ports: [
                    {
                      p2pPort: PORT_PREFIX * 1000 + 100,
                      wsPort: PORT_PREFIX * 1000 + 102,
                      rpcPort: PORT_PREFIX * 1000 + 101,
                    },
                  ],
                },
              ],
              relayPorts: [],
            };
        // Context is given prior to this assignement, so doing
        // context = init.context will fail because it replace the variable;

        context._polkadotApiParachains = [];
        context._polkadotApiRelaychains = [];
        context._web3Providers = [];
        context.blockNumber = 0;

        context.createWeb3 = async (protocol: "ws" | "http" = "http") => {
          const provider =
            protocol == "ws"
              ? await provideWeb3Api(`ws://localhost:${init.paraPorts[0].ports[0].wsPort}`)
              : await provideWeb3Api(`http://localhost:${init.paraPorts[0].ports[0].rpcPort}`);
          context._web3Providers.push((provider as any)._provider);
          return provider;
        };
        context.createEthers = async () =>
          provideEthersApi(`http://localhost:${init.paraPorts[0].ports[0].rpcPort}`);
        context.createPolkadotApiParachain = async (parachainNumber: number) => {
          const promise = providePolkadotApi(init.paraPorts[parachainNumber].ports[0].wsPort);
          context._polkadotApiParachains.push({
            parachainId: init.paraPorts[parachainNumber].parachainId,
            apis: [await promise],
          });
          return promise;
        };
        context.createPolkadotApiParachains = async () => {
          const apiPromises = await Promise.all(
            init.paraPorts.map(async (parachain: ParachainPorts) => {
              return {
                parachainId: parachain.parachainId,
                apis: await Promise.all(
                  parachain.ports.map(async (ports: NodePorts) => {
                    return providePolkadotApi(ports.wsPort);
                  })
                ),
              };
            })
          );
          // We keep track of the polkadotApis to close them at the end of the test
          context._polkadotApiParachains = apiPromises;
          await Promise.all(
            apiPromises.map(async (promises) =>
              Promise.all(promises.apis.map((promise) => promise.isReady))
            )
          );
          // Necessary hack to allow polkadotApi to finish its internal metadata loading
          // apiPromise.isReady unfortunately doesn't wait for those properly
          await new Promise((resolve) => {
            setTimeout(resolve, 100);
          });

          return apiPromises[0].apis[0];
        };
        context.createPolkadotApiRelaychains = async () => {
          const apiPromises = await Promise.all(
            init.relayPorts.map(async (ports: NodePorts) => {
              return await providePolkadotApi(ports.wsPort, true);
            })
          );
          // We keep track of the polkadotApis to close them at the end of the test
          context._polkadotApiRelaychains = apiPromises;
          await Promise.all(apiPromises.map((promise) => promise.isReady));
          // Necessary hack to allow polkadotApi to finish its internal metadata loading
          // apiPromise.isReady unfortunately doesn't wait for those properly
          await new Promise((resolve) => {
            setTimeout(resolve, 100);
          });

          return apiPromises[0];
        };

        let pendingCallbacks: {
          blockNumber: number;
          resolve: (blockNumber: number) => void;
        }[] = [];
        const subBlocks = async (api: ApiPromise) => {
          return api.rpc.chain.subscribeNewHeads(async (header) => {
            context.blockNumber = header.number.toNumber();
            if (context.blockNumber == 0) {
              console.log(
                `Start listening for new blocks. Production will start in ${chalk.red(`1 minute`)}`
              );
            }
            debug(`New block: #${context.blockNumber}`);

            let i = pendingCallbacks.length;
            while (i--) {
              const pendingCallback = pendingCallbacks[i];
              if (pendingCallback.blockNumber <= context.blockNumber) {
                pendingCallbacks.splice(i, 1);
                pendingCallback.resolve(context.blockNumber);
              }
            }
          });
        };

        context.polkadotApiParaone = await context.createPolkadotApiParachains();
        subBlocks(context.polkadotApiParaone);

        context.waitBlocks = async (count: number) => {
          return new Promise<number>((resolve) => {
            pendingCallbacks.push({
              blockNumber: (context.blockNumber || 0) + count,
              resolve,
            });
          });
        };

        context.upgradeRuntime = async (
          from: KeyringPair,
          runtimeName: "moonbase" | "moonriver" | "moonbeam",
          runtimeVersion: "local" | string,
          { waitMigration = true, useGovernance = false } = {
            waitMigration: true,
            useGovernance: false,
          }
        ) => {
          const api = context.polkadotApiParaone;
          return new Promise<number>(async (resolve, reject) => {
            try {
              const code = fs
                .readFileSync(await getRuntimeWasm(runtimeName, runtimeVersion))
                .toString();

              const existingCode = await api.rpc.state.getStorage(":code");
              if (existingCode.toString() == code) {
                reject(
                  `Runtime upgrade with same code: ${existingCode.toString().slice(0, 20)} vs ${code
                    .toString()
                    .slice(0, 20)}`
                );
              }

              let nonce = (await api.rpc.system.accountNextIndex(from.address)).toNumber();

              if (useGovernance) {
                // We just prepare the proposals
                let proposal = api.tx.parachainSystem.authorizeUpgrade(blake2AsHex(code));
                let encodedProposal = proposal.method.toHex();
                let encodedHash = blake2AsHex(encodedProposal);

                // Check if already in governance
                const preImageExists = await api.query.democracy.preimages(encodedHash);
                if (preImageExists.isSome && preImageExists.unwrap().isAvailable) {
                  process.stdout.write(`Preimage ${encodedHash} already exists !\n`);
                } else {
                  process.stdout.write(
                    `Registering preimage (${sha256(Buffer.from(code))} [~${Math.floor(
                      code.length / 1024
                    )} kb])...`
                  );
                  await api.tx.democracy
                    .notePreimage(encodedProposal)
                    .signAndSend(from, { nonce: nonce++ });
                  process.stdout.write(`✅\n`);
                }

                // Check if already in referendum
                const referendum = await api.query.democracy.referendumInfoOf.entries();
                const referendaIndex = referendum
                  .filter(
                    (ref) =>
                      ref[1].unwrap().isOngoing &&
                      ref[1].unwrap().asOngoing.proposalHash.toHex() == encodedHash
                  )
                  .map((ref) =>
                    api.registry.createType("u32", ref[0].toU8a().slice(-4)).toNumber()
                  )?.[0];
                if (referendaIndex !== null && referendaIndex !== undefined) {
                  process.stdout.write(`Vote for upgrade already in referendum, cancelling it.\n`);
                  await cancelReferendaWithCouncil(api, referendaIndex);
                }
                await executeProposalWithCouncil(api, encodedHash);

                // Needs to retrieve nonce after those governance calls
                nonce = (await api.rpc.system.accountNextIndex(from.address)).toNumber();
                process.stdout.write(`Enacting authorized upgrade...`);
                await api.tx.parachainSystem
                  .enactAuthorizedUpgrade(code)
                  .signAndSend(from, { nonce: nonce++ });
                process.stdout.write(`✅\n`);
              } else {
                process.stdout.write(
                  `Sending sudo.setCode (${sha256(Buffer.from(code))} [~${Math.floor(
                    code.length / 1024
                  )} kb])...`
                );
                await api.tx.sudo
                  .sudoUncheckedWeight(await api.tx.system.setCodeWithoutChecks(code), 1)
                  .signAndSend(from, { nonce: nonce++ });
                process.stdout.write(`✅\n`);
              }

              process.stdout.write(`Waiting to apply new runtime (${chalk.red(`~4min`)})...`);
              let isInitialVersion = true;
              const unsub = await api.rpc.state.subscribeRuntimeVersion(async (version) => {
                if (!isInitialVersion) {
                  const blockNumber = context.blockNumber;
                  console.log(
                    `✅ [${version.implName}-${version.specVersion} ${existingCode
                      .toString()
                      .slice(0, 6)}...] [#${blockNumber}]`
                  );
                  unsub();
                  const newCode = await api.rpc.state.getStorage(":code");
                  if (newCode.toString() != code) {
                    reject(
                      `Unexpected new code: ${newCode.toString().slice(0, 20)} vs ${code
                        .toString()
                        .slice(0, 20)}`
                    );
                  }
                  if (waitMigration) {
                    // Wait for next block to have the new runtime applied
                    await context.waitBlocks(1);
                  }
                  resolve(blockNumber);
                }
                isInitialVersion = false;
              });
            } catch (e) {
              console.error(`Failed to setCode`);
              reject(e);
            }
          });
        };
        context.web3 = await context.createWeb3();
        context.ethers = await context.createEthers();
        debug(
          `Setup ready [${/:([0-9]+)$/.exec((context.web3.currentProvider as any).host)[1]}] for ${
            this.currentTest.title
          }`
        );
      } catch (e) {
        console.error(`Failed to start nodes !!!`);
        console.error(e);
        process.exit(1);
      }
    });

    after(async function () {
      await Promise.all(context._web3Providers.map((p) => p.disconnect()));
      await Promise.all(
        context._polkadotApiParachains.map(
          async (ps) => await Promise.all(ps.apis.map((p) => p.disconnect()))
        )
      );
      await Promise.all(context._polkadotApiRelaychains.map((p) => p.disconnect()));

      if (!DEBUG_MODE) {
        await stopParachainNodes();
        await new Promise((resolve) => {
          // TODO: Replace Sleep by actually checking the process has ended
          setTimeout(resolve, 1000);
        });
      }
    });

    cb(context);
  });
}
