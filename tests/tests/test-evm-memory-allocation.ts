import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("Excessive memory allocation", (context) => {
  it("should fail with OOG", async () => {
    // this tests a security vulnerability in our EVM which was patched in May 2021 or so.
    // The vulnerability allowed contract code to request an extremely large amount of memory,
    // causing a node to crash.
    //
    // fixed by:
    // https://github.com/rust-blockchain/evm/commit/19ade858c430ab13eb562764a870ac9f8506f8dd

    /*
    const bytecode = 
      [65, 65, 4, 97, 89, 134, 65, 65,
        65, 65, 65, 52, 57, 51, 52, 51,
        70, 70, 1, 0, 0, 0, 40, 249,
        0, 224, 111, 1, 0, 0, 0, 247,
        30, 1, 0, 0, 0, 0, 0, 0];
        */

    const value = "0x" + (993452714685890559).toString(16);

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: null,
        value: value,
        gas: "0x100000",
        gasPrice: 1_000_000_000,
        data: "0x4141046159864141414141343933343346460100000028F900E06F01000000F71E01000000000000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);
    await context.createBlock();

    const receipt = await context.web3.eth.getTransactionReceipt(txResults.result);
    expect(receipt.status).to.be.false;
  });
});
