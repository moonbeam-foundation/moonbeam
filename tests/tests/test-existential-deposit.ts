import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

describeWithMoonbeam("Moonbeam RPC (Existential Deposit)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";

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
