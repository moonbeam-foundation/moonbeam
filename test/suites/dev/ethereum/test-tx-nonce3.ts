import { Web3, describeSuite, expect, beforeAll } from "@moonwall/cli";
import {
  EthTester,
  GERALD_ADDRESS,
  GERALD_PRIVATE_KEY,
  ALITH_PRIVATE_KEY,
  ALITH_ADDRESS,
} from "@moonwall/util";

describeSuite({
  id: "D04",
  title: "Ethereum Nonce: fully transferred account",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let web3: Web3;
    let ethTester: EthTester;

    beforeAll(async () => {
      web3 = context.web3();
      ethTester = new EthTester(web3, ALITH_PRIVATE_KEY, log);

      const initialBalance = await web3.eth.getBalance(GERALD_ADDRESS);

      // Transfer all the balance to another account
      const signedTransfer = await ethTester.genSignedTransfer(
        ALITH_ADDRESS,
        initialBalance - 21000n * BigInt(web3.utils.toWei("10", "gwei")),
        {
          gas: web3.utils.toHex(21000),
          privateKey: GERALD_PRIVATE_KEY,
        }
      );

      await context.createBlock(signedTransfer);

      // Verify the account balance is empty
      expect(await web3.eth.getBalance(GERALD_ADDRESS)).to.eq(0n);
    });

    it({
      id: "E01",
      title: "should keep its nonce to 1",
      modifier: "only",
      test: async function () {
        web3;
        expect(await web3.eth.getTransactionCount(GERALD_ADDRESS)).to.eq(1n);
      },
    });
  },
});
