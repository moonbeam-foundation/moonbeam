import { Web3,describeSuite, expect, beforeAll, Signer } from "@moonsong-labs/moonwall-cli";
import {
  alith,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  EthTester,
  GERALD_ADDRESS,
  GERALD_PRIVATE_KEY,
  ALITH_PRIVATE_KEY,
} from "@moonsong-labs/moonwall-util";

describeSuite({
  id: "D02",
  title: "Ethereum Nonce: initial values",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let web3: Web3;
    let ethTester: EthTester;
    let api: Signer;
    beforeAll(() => {
      api = context.ethersSigner();
      web3 = context.web3();
      ethTester = new EthTester(web3, ALITH_PRIVATE_KEY, log);
    });

    it({
      id: "E01",
      title: "should be at 0 for non existing account",
      test: async function () {
        expect(
          await web3.eth.getTransactionCount("0x0000000000000000000000000000000000001234")
        ).to.eq(0n);
      },
    });

    it({
      id: "E02",
      title: "should be at 0 for genesis accounts",
      test: async function () {
        expect(await web3.eth.getTransactionCount(alith.address)).to.eq(0n);
        expect(await web3.eth.getTransactionCount(baltathar.address)).to.eq(0n);
      },
    });

    it({
      id: "E03",
      title: "should be at 0 before the block is created",
      modifier: "skip", // pending https://github.com/PureStake/moonbeam/issues/2184
      test: async function () {
        // Adds the transaction to the pool but
        // doesn't produce the block.
        await ethTester.sendSignedTransaction(
          ethTester.genSignedTransfer(alith.address, Web3.utils.toWei("1", "ether"), {
            privateKey: BALTATHAR_PRIVATE_KEY,
          })
        );
        const pendingTxs = await ethTester.web3.eth.getPendingTransactions();
        expect(pendingTxs.length).to.eq(1);
        expect(await web3.eth.getTransactionCount(alith.address)).to.eq(0n);
      },
    });
  },
});