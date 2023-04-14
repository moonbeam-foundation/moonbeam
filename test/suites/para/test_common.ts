import { expect, describeSuite, beforeAll, ApiPromise } from "@moonwall/cli";
import { Signer, ethers } from "ethers";
import {
  ALITH_ADDRESS,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  alith,
  charleth,
} from "@moonwall/util";
import "@moonbeam-network/api-augment";

describeSuite({
  id: "ZMB",
  title: "Zombie Test Suite",
  foundationMethods: "zombie",
  testCases: function ({ it, context, log }) {
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;
    let ethersSigner: Signer;

    beforeAll(() => {
      paraApi = context.polkadotJs({ type: "moon" });
      relayApi = context.polkadotJs({ type: "polkadotJs" });
      ethersSigner = context.ethersSigner();

      const relayNetwork = relayApi.consts.system.version.specName.toString();
      expect(relayNetwork, "Relay API incorrect").to.contain("rococo");
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
      title: "Can connect to parachain and execute a transaction",
      timeout: 60000,
      test: async function () {
        const balBefore = (await paraApi.query.system.account(BALTATHAR_ADDRESS)).data.free;

        log("Please wait, this will take at least 30s for transaction to complete");

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
      id: "T03",
      title: "Tags are present on emulated Ethereum blocks",
      test: async function () {
        expect(
          (await ethersSigner.provider.getBlock("safe")).number,
          "Safe tag is not present"
        ).to.be.greaterThan(0);
        expect(
          (await ethersSigner.provider.getBlock("finalized")).number,
          "Finalized tag is not present"
        ).to.be.greaterThan(0);
        expect(
          (await ethersSigner.provider.getBlock("latest")).number,
          "Latest tag is not present"
        ).to.be.greaterThan(0);
        // log(await ethersSigner.provider.getTransactionCount(ALITH_ADDRESS, "latest"));
        // await context
        //   .ethersSigner()
        //   .sendTransaction({ to: BALTATHAR_ADDRESS, value: ethers.parseEther("1") });
        // log(await ethersSigner.provider.getTransactionCount(ALITH_ADDRESS, "pending"));
      },
    });
  },
});
