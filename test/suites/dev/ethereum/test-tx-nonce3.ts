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
} from "@moonsong-labs/moonwall-util";
import { WebSocketProvider, parseUnits } from "ethers";
import { ApiPromise } from "@polkadot/api";

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
