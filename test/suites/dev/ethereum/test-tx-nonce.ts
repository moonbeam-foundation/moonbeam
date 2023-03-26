import Web3 from "web3";

import { describeSuite, expect, beforeAll } from "@moonsong-labs/moonwall-cli";
import {
  alith,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  EthTester,
  GERALD_ADDRESS,
  GERALD_PRIVATE_KEY,
  ALITH_PRIVATE_KEY,
  printEvents,
  printExtrinsicWithEvent,
} from "@moonsong-labs/moonwall-util";
import { WebSocketProvider, parseUnits } from "ethers";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D02",
  title: "Ethereum Nonce: initial values",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let web3: Web3;
    let ethTester: EthTester;
    let api: WebSocketProvider;
    beforeAll(() => {
      api = context.getEthers();
      web3 = context.getWeb3();
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

describeSuite({
  id: "D03",
  title: "Ethereum Nonce: logic",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let web3: Web3;
    let ethTester: EthTester;
    let api: WebSocketProvider;

    // Create the block with Baltathar transaction
    beforeAll(async () => {
      api = context.getEthers();
      web3 = context.getWeb3();
      ethTester = new EthTester(web3, ALITH_PRIVATE_KEY, log);

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

describeSuite({
  id: "D04",
  title: "Ethereum Nonce: fully transferred account",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let web3: Web3;
    let ethTester: EthTester;
    let api: WebSocketProvider;

    beforeAll(async () => {
      api = context.getEthers();
      web3 = context.getWeb3();
      ethTester = new EthTester(web3, ALITH_PRIVATE_KEY, log);

      const initialBalance = await web3.eth.getBalance(GERALD_ADDRESS);

      // Transfer all the balance to another account
      // TODO: This fails because the tx is not sent it seems, not sure why.
      await context.createBlock(
        ethTester.genSignedTransfer(
          alith.address,
          initialBalance - 21000n * parseUnits("10", "gwei"),
          {
            gas: parseUnits("10", "gwei"),
            privateKey: GERALD_PRIVATE_KEY,
          }
        )
      );

      // Verify the account balance is empty
      expect(await web3.eth.getBalance(GERALD_ADDRESS)).to.eq("0");
    });

    it({
      id: "E01",
      title: "should keep its nonce to 1",
      modifier: "only",
      test: async function () {
        expect(await web3.eth.getTransactionCount(GERALD_ADDRESS)).to.eq(1n);
      },
    });
  },
});
