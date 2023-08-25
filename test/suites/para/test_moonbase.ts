import "@moonbeam-network/api-augment";
import { MoonwallContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, charleth } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { Wallet, ethers } from "ethers";
import fs from "node:fs";

describeSuite({
  id: "ZAN",
  title: "Zombie AlphaNet Upgrade Test",
  foundationMethods: "zombie",
  testCases: function ({ it, context, log }) {
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;

    beforeAll(async () => {
      paraApi = context.polkadotJs("parachain");
      relayApi = context.polkadotJs("relaychain");

      const relayNetwork = relayApi.consts.system.version.specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("rococo");

      const paraNetwork = paraApi.consts.system.version.specName.toString();
      expect(paraNetwork, "Para API incorrect").to.contain("moonbase");

      const currentBlock = (await paraApi.rpc.chain.getBlock()).block.header.number.toNumber();
      expect(currentBlock, "Parachain not producing blocks").to.be.greaterThan(0);
    }, 120000);

    it({
      id: "T01",
      title: "Blocks are being produced on parachain",
      test: async function () {
        const blockNum = (await paraApi.rpc.chain.getBlock()).block.header.number.toNumber();
        expect(blockNum).to.be.greaterThan(0);
      },
    });

    it({
      id: "T02",
      title: "Chain can be upgraded",
      timeout: 600000,
      test: async function () {
        const blockNumberBefore = (
          await paraApi.rpc.chain.getBlock()
        ).block.header.number.toNumber();
        const currentCode = (await paraApi.rpc.state.getStorage(":code")) as any;
        const codeString = currentCode.toString();

        const wasm = fs.readFileSync(MoonwallContext.getContext().rtUpgradePath!);
        const rtHex = `0x${wasm.toString("hex")}`;

        if (rtHex === codeString) {
          log("Runtime already upgraded, skipping test");
          return;
        } else {
          log("Runtime not upgraded, proceeding with test");
          log("Current runtime hash: " + rtHex.slice(0, 10) + "..." + rtHex.slice(-10));
          log("New runtime hash: " + codeString.slice(0, 10) + "..." + codeString.slice(-10));
        }

        await context.upgradeRuntime({ logger: log });
        await context.waitBlock(2);
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
      timeout: 60000,
      test: async function () {
        const balBefore = (await paraApi.query.system.account(BALTATHAR_ADDRESS)).data.free;

        log("Please wait, this will take at least 30s for transaction to complete");

        context.waitBlock(5);

        await new Promise((resolve) => {
          paraApi.tx.balances
            .transfer(BALTATHAR_ADDRESS, ethers.parseEther("2"))
            .signAndSend(charleth, ({ status, events }) => {
              if (status.isInBlock) {
                log("Transaction is in block");
              }
              if (status.isFinalized) {
                log("Transaction is finalized!");
                resolve(events);
              }
            });
        });

        const balAfter = (await paraApi.query.system.account(BALTATHAR_ADDRESS)).data.free;
        expect(balBefore.lt(balAfter)).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "Tags are present on emulated Ethereum blocks",
      test: async function () {
        expect(
          (await context.ethers().provider!.getBlock("safe"))!.number,
          "Safe tag is not present"
        ).to.be.greaterThan(0);
        expect(
          (await context.ethers().provider!.getBlock("finalized"))!.number,
          "Finalized tag is not present"
        ).to.be.greaterThan(0);
        expect(
          (await context.ethers().provider!.getBlock("latest"))!.number,
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
