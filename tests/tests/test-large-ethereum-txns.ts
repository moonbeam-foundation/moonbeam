import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../util/constants";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("Large Ethereum Transactions", (context) => {

  // function to generate a junk transaction with a specified data size
  const generateLargeTxn = async (size) => {
    const byte = "FF";
    const data = "0x"+ byte.repeat(size);

    return await context.web3.eth.accounts.signTransaction(
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
  }

  // TODO: I'm not sure where this 2000 came from...
  const max_size = ((EXTRINSIC_GAS_LIMIT - 21000) / 16) - 2000;

  it("should accept txns up to known size", async function () {
    expect(max_size).to.equal(808875); // our max Ethereum TXN size in bytes

    const tx = await generateLargeTxn(max_size);
    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);
    await context.createBlock();
    const receipt = await context.web3.eth.getTransactionReceipt(txResults.result);

    expect(receipt.status).to.be.false; // this txn is nonsense, but the RPC should accept it
  });

  it("should reject txns which are too large to pay for", async function () {
    const tx = await generateLargeTxn(max_size + 1);
    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    // TODO: how can we get a signed txn that is too large? web3 will reject it because it knows it
    //       will be too large...
    // NOTE: I was able to hack this check out of web3 and verified that our node rejected the txn
    //       (resulting in the error message below). 

    // RPC should outright reject this txn -- this is important because it prevents it from being
    // gossipped, thus preventing potential for spam
    expect(txResults).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      error: { message: "gas limit reached", code: -32603 },
    });
  });
});
