import "@moonbeam-network/api-augment";
import { MoonwallContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, alith, charleth } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { ethers } from "ethers";
import fs from "node:fs";

describeSuite({
  id: "Z01",
  title: "Zombie AlphaNet Upgrade Test",
  foundationMethods: "zombie",
  testCases: ({ it, context, log }) => {
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;

    beforeAll(async () => {
      paraApi = context.polkadotJs("parachain");
      relayApi = context.polkadotJs("relaychain");

      const currentBlock = (await paraApi.rpc.chain.getBlock()).block.header.number.toNumber();
      expect(currentBlock, "Parachain not producing blocks").to.be.greaterThan(0);
    }, 120000);

    it({
      id: "T01",
      title: "Blocks are being produced on parachain",
      test: async () => {
        const blockNum = (await paraApi.rpc.chain.getBlock()).block.header.number.toNumber();
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

        const blockNumberBefore = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();

        await paraApi.tx.parachainSystem.enactAuthorizedUpgrade(rtHex).signAndSend(alith);

        await context.waitBlock(15);

        const rtafter = paraApi.consts.system.version.specVersion.toNumber();
        expect(rtafter).to.be.greaterThan(rtBefore);

        log(`RT upgrade has increased specVersion from ${rtBefore} to ${rtafter}`);

        const blockNumberAfter = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();
        log(`Before: #${blockNumberBefore}, After: #${blockNumberAfter}`);
        expect(blockNumberAfter, "Block number did not increase").to.be.greaterThan(
          blockNumberBefore
        );
      },
    });

    it({
      id: "T03",
      title: "Can connect to parachain and execute a transaction",
      timeout: 600000,
      test: async () => {
        const balBefore = (await paraApi.query.system.account(BALTATHAR_ADDRESS)).data.free;

        log("Please wait, this will take at least 30s for transaction to complete");

        context.waitBlock(5);

        await new Promise((resolve, reject) => {
          paraApi.tx.balances
            .transferAllowDeath(BALTATHAR_ADDRESS, ethers.parseEther("2"))
            .signAndSend(charleth, ({ status, events }) => {
              if (status.isInBlock) {
                log("Transaction is in block");
              }
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
                log(status.toHuman());
                log(events.map(e => e.toHuman()));
                throw new Error("Transaction failed");
              }
            });
        });

        const balAfter = (await paraApi.query.system.account(BALTATHAR_ADDRESS)).data.free;
        expect(
          balBefore.lt(balAfter),
          `${balBefore.toHuman()} is not less than ${balAfter.toHuman()}`
        ).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "Tags are present on emulated Ethereum blocks",
      test: async () => {
        expect(
          (await context.ethers().provider?.getBlock("safe"))?.number,
          "Safe tag is not present"
        ).to.be.greaterThan(0);
        expect(
          (await context.ethers().provider?.getBlock("finalized"))?.number,
          "Finalized tag is not present"
        ).to.be.greaterThan(0);
        expect(
          (await context.ethers().provider?.getBlock("latest"))?.number,
          "Latest tag is not present"
        ).to.be.greaterThan(0);
        // log(await ethersSigner.provider.getTransactionCount(ALITH_ADDRESS, "latest"));
        // await context
        //   .ethers()
        //   .sendTransaction({ to: BALTATHAR_ADDRESS, value: ethers.parseEther("1") });
        // log(await ethersSigner.provider.getTransactionCount(ALITH_ADDRESS, "pending"));
      },
    });
  },
});
