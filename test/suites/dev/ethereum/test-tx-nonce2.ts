import { Web3, describeSuite, expect, beforeAll } from "@moonwall/cli";
import {
  alith,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  EthTester,
  ALITH_PRIVATE_KEY,
} from "@moonwall/util";

describeSuite({
  id: "D03",
  title: "Ethereum Nonce: logic",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let web3: Web3;
    let ethTester: EthTester;

    // Create the block with Baltathar transaction
    beforeAll(async () => {
      web3 = context.web3();
      ethTester = new EthTester(web3, ALITH_PRIVATE_KEY, log, "EIP1559");

      // Create a block to increase the nonce of baltathar
      await context.createBlock(
        ethTester.genSignedTransfer(alith.address, Web3.utils.toWei("1", "ether"), {
          privateKey: BALTATHAR_PRIVATE_KEY,
        })
      );
    });

    it({
      id: "E01",
      title: "should increase the sender nonce to 1",
      test: async function () {
        expect(await web3.eth.getTransactionCount(baltathar.address)).to.eq(1n);
      },
    });

    it({
      id: "E02",
      title: "should keep the received nonce at 0",
      test: async function () {
        expect(await web3.eth.getTransactionCount(alith.address)).to.eq(0n);
      },
    });
  },
});
