import "@moonbeam-network/api-augment";
import {
  BALTATHAR_ADDRESS,
  MoonwallContext,
  alith,
  beforeAll,
  charleth,
  describeSuite,
  expect,
} from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import { ethers } from "ethers";
import fs from "node:fs";

describeSuite({
  id: "Z01",
  title: "Zombienet - Runtime Upgrade Test",
  foundationMethods: "zombie",
  testCases: ({ it, context, log }) => {
    let paraApi: ApiPromise;

    beforeAll(async () => {
      paraApi = context.polkadotJs("parachain");

      const currentBlock = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      expect(currentBlock, "Parachain not producing blocks").to.be.greaterThan(0);
    }, 120000);

    it({
      id: "T01",
      title: "Blocks are being produced on parachain",
      test: async () => {
        const blockNum = (await paraApi.rpc.chain.getHeader()).number.toNumber();
        expect(blockNum).to.be.greaterThan(0);
      },
    });

    it({
      id: "T02",
      title: "Chain can be upgraded",
      timeout: 600000,
      test: async () => {
        const currentCode = (await paraApi.rpc.state.getStorage(":code")) as any;
        const codeString = currentCode.toString();
        const upgradePath = (await MoonwallContext.getContext()).rtUpgradePath;

        const rtBefore = paraApi.consts.system.version.specVersion.toNumber();

        if (!upgradePath) {
          throw new Error("Runtime upgrade path not set");
        }

        const wasm = fs.readFileSync(upgradePath);
        const rtHex = `0x${wasm.toString("hex")}`;

        if (rtHex === codeString) {
          log("Runtime already upgraded, skipping test");
          return;
        }

        log("Runtime not upgraded, proceeding with test");
        log(`Current runtime hash: ${rtHex.slice(0, 10)}...${rtHex.slice(-10)}`);
        log(`New runtime hash: ${codeString.slice(0, 10)}...${codeString.slice(-10)}`);

        const blockNumberBefore = (await paraApi.rpc.chain.getHeader()).number.toNumber();

        await paraApi.tx.system.applyAuthorizedUpgrade(rtHex).signAndSend(alith);

        await context.waitBlock(15);

        const rtafter = paraApi.consts.system.version.specVersion.toNumber();
        expect(rtafter).to.be.greaterThan(rtBefore);

        log(`RT upgrade has increased specVersion from ${rtBefore} to ${rtafter}`);

        const blockNumberAfter = (await paraApi.rpc.chain.getHeader()).number.toNumber();
        log(`Before: #${blockNumberBefore}, After: #${blockNumberAfter}`);
        expect(blockNumberAfter, "Block number did not increase").to.be.greaterThan(
          blockNumberBefore
        );
      },
    });

    it({
      id: "T03",
      title: "Can connect to parachain and execute a transaction",
      timeout: 240000,
      test: async () => {
        const balBefore = (
          await paraApi.query.system.account(BALTATHAR_ADDRESS)
        ).data.free.toBigInt();

        log("Please wait, this will take at least 30s for transaction to complete");

        await new Promise((resolve, reject) => {
          paraApi.tx.balances
            .transferAllowDeath(BALTATHAR_ADDRESS, ethers.parseEther("2"))
            .signAndSend(charleth, ({ status, events }) => {
              log(`Transaction status: ${JSON.stringify(status.toHuman())}`);

              if (status.isFinalized) {
                log("Transaction is finalized!");
                resolve(events);
              }

              if (
                status.isDropped ||
                status.isInvalid ||
                status.isUsurped ||
                status.isFinalityTimeout
              ) {
                reject("transaction failed!");
                throw new Error("Transaction failed");
              }
            });
        });

        const balAfter = (
          await paraApi.query.system.account(BALTATHAR_ADDRESS)
        ).data.free.toBigInt();
        expect(balBefore < balAfter, `${balBefore} is not less than ${balAfter}`).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "Tags are present on emulated Ethereum blocks",
      test: async () => {
        const waitForTag = async (tag: "safe" | "finalized", maxAttempts = 15) => {
          for (let attempt = 0; attempt < maxAttempts; attempt++) {
            const block = await context.ethers().provider?.getBlock(tag);
            if (typeof block?.number === "number" && block.number > 0) {
              return block.number;
            }
            await context.waitBlock(1);
          }
          return undefined;
        };

        expect(await waitForTag("safe"), "Safe tag is not present").to.be.greaterThan(0);
        expect(await waitForTag("finalized"), "Finalized tag is not present").to.be.greaterThan(0);
        const latestBlock = await context.ethers().provider?.getBlock("latest");
        expect(latestBlock, "Latest tag is not present").to.not.equal(null);
        expect(latestBlock?.number, "Latest tag is not present").to.be.greaterThan(0);
      },
    });
  },
});
