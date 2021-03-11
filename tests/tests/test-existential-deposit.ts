import { expect } from "chai";
import { step } from "mocha-steps";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_ACCOUNT } from "./constants";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

describeWithMoonbeam("Moonbeam RPC (Existential Deposit)", `simple-specs.json`, (context) => {
  step("Account is not reaped on zero balance", async function () {
    const balance = await context.web3.eth.getBalance(GENESIS_ACCOUNT);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: balance,
        gasPrice: "0x00",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    expect(parseInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT))).to.eq(0);
    expect(await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT)).to.eq(1);
  });
});
