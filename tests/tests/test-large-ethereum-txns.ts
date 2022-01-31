import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../util/constants";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("Large Ethereum Transactions", (context) => {
  it("should reject txns which are too large to pay for", async function () {

    // generate a txn blob that should cause a per-byte fee in gas that would be larger than the
    // block gas limit.
    const byte = "FF";
    const data = "0x"+ byte.repeat(1024 * 1024 * 4);
    // const data = "0x"+ byte.repeat(64);

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: null,
        value: "0x0",
        gas: EXTRINSIC_GAS_LIMIT,
        gasPrice: 1_000_000_000,
        data: data,
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    expect(txResults).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      error: { message: "gas limit reached", code: -32603 },
    });
  });
});
