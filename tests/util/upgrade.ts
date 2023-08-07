import fs from "fs";
import chalk from "chalk";

import type { WeightV2 } from "@polkadot/types/interfaces";

import { ApiPromise } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { blake2AsHex } from "@polkadot/util-crypto";
import { sha256 } from "ethers/lib/utils";

import { getRuntimeWasm } from "./binaries";
import { cancelReferendaWithCouncil, executeProposalWithCouncil } from "./governance";
import { alith } from "./accounts";

export interface UpgradePreferences {
  runtimeName: "moonbase" | "moonriver" | "moonbeam";
  runtimeTag: "local" | string;
  from?: KeyringPair;
  waitMigration?: boolean;
  useGovernance?: boolean;
}

export async function upgradeRuntime(api: ApiPromise, preferences: UpgradePreferences) {
  const options = {
    from: alith,
    waitMigration: true,
    useGovernance: false,
    ...preferences,
  };
  return new Promise<number>(async (resolve, reject) => {
    try {
      const code = fs
        .readFileSync(await getRuntimeWasm(options.runtimeName, options.runtimeTag))
        .toString();

      const existingCode = await api.rpc.state.getStorage(":code");
      if (existingCode.toString() == code) {
        reject(
          `Runtime upgrade with same code: ${existingCode.toString().slice(0, 20)} vs ${code
            .toString()
            .slice(0, 20)}`
        );
      }

      let nonce = (await api.rpc.system.accountNextIndex(options.from.address)).toNumber();

      if (options.useGovernance) {
        // TODO: remove support for old style after all chains upgraded to 2400+
        let proposal =
          api.consts.system.version.specVersion.toNumber() >= 2400
            ? (api.tx.parachainSystem as any).authorizeUpgrade(blake2AsHex(code), false)
            : (api.tx.parachainSystem as any).authorizeUpgrade(blake2AsHex(code));
        let encodedProposal = proposal.method.toHex();
        let encodedHash = blake2AsHex(encodedProposal);

        // Check if already in governance
        const preImageExists =
          api.query.preimage && (await api.query.preimage.statusFor(encodedHash));
        const democracyPreImageExists =
          !api.query.preimage && ((await api.query.democracy.preimages(encodedHash)) as any);

        if (api.query.preimage && preImageExists.isSome && preImageExists.unwrap().isRequested) {
          process.stdout.write(`Preimage ${encodedHash} already exists !\n`);
        } else if (
          // TODO: remove support for democracy preimage support after 2000
          !api.query.preimage &&
          democracyPreImageExists.isSome &&
          democracyPreImageExists.unwrap().isAvailable
        ) {
          process.stdout.write(`Preimage ${encodedHash} already exists !\n`);
        } else {
          process.stdout.write(
            `Registering preimage (${sha256(Buffer.from(code))} [~${Math.floor(
              code.length / 1024
            )} kb])...`
          );
          if (api.query.preimage) {
            await api.tx.preimage
              .notePreimage(encodedProposal)
              .signAndSend(options.from, { nonce: nonce++ });
          } else {
            // TODO: remove support for democracy after 2000
            await api.tx.democracy
              .notePreimage(encodedProposal)
              .signAndSend(options.from, { nonce: nonce++ });
          }
          process.stdout.write(`✅\n`);
        }

        // Check if already in referendum
        const referendum = await api.query.democracy.referendumInfoOf.entries();
        // TODO: remove support for democracy after 2000
        const referendaIndex = api.query.preimage
          ? referendum
              .filter(
                (ref) =>
                  ref[1].unwrap().isOngoing &&
                  ref[1].unwrap().asOngoing.proposal.isLookup &&
                  ref[1].unwrap().asOngoing.proposal.asLookup.hash.toHex() == encodedHash
              )
              .map((ref) =>
                api.registry.createType("u32", ref[0].toU8a().slice(-4)).toNumber()
              )?.[0]
          : referendum
              .filter(
                (ref) =>
                  ref[1].unwrap().isOngoing &&
                  (ref[1].unwrap().asOngoing as any).proposalHash.toHex() == encodedHash
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
        nonce = (await api.rpc.system.accountNextIndex(options.from.address)).toNumber();
        process.stdout.write(`Enacting authorized upgrade...`);
        await api.tx.parachainSystem
          .enactAuthorizedUpgrade(code)
          .signAndSend(options.from, { nonce: nonce++ });
        process.stdout.write(`✅\n`);
      } else {
        process.stdout.write(
          `Sending sudo.setCode (${sha256(Buffer.from(code))} [~${Math.floor(
            code.length / 1024
          )} kb])...`
        );
        const isWeightV1 = !api.registry.createType<WeightV2>("Weight").proofSize;
        await api.tx.sudo
          .sudoUncheckedWeight(
            await api.tx.system.setCodeWithoutChecks(code),
            isWeightV1
              ? "1"
              : {
                  proofSize: 1,
                  refTime: 1,
                }
          )
          .signAndSend(options.from, { nonce: nonce++ });
        process.stdout.write(`✅\n`);
      }

      process.stdout.write(`Waiting to apply new runtime (${chalk.red(`~4min`)})...`);
      let isInitialVersion = true;
      const unsub = await api.rpc.state.subscribeRuntimeVersion(async (version) => {
        if (!isInitialVersion) {
          const blockNumber = (await api.rpc.chain.getHeader()).number.toNumber();
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
          if (options.waitMigration) {
            const blockToWait = (await api.rpc.chain.getHeader()).number.toNumber() + 1;
            await new Promise(async (resolve) => {
              const subBlocks = await api.rpc.chain.subscribeNewHeads(async (header) => {
                if (header.number.toNumber() == blockToWait) {
                  subBlocks();
                  resolve(blockToWait);
                }
              });
            });
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
}
