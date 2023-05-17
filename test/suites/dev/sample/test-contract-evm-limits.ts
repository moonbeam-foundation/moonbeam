import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, ALITH_PRIVATE_KEY } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Contract - Excessive memory allocation", (context) => {
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
        from: alith.address,
        to: null,
        value: value,
        gas: "0x100000",
        gasPrice: 10_000_000_000,
        data: "0x4141046159864141414141343933343346460100000028F900E06F01000000F71E01000000000000",
      },
      ALITH_PRIVATE_KEY
    );

    const txResults = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      tx.rawTransaction,
    ]);

    // before this was patched, attempting to execute this transaction would cause an extremely
    // large memory allocation and the node would crash.
    await context.createBlock();

    const receipt = await context.web3.eth.getTransactionReceipt(txResults.result);
    expect(receipt.status).to.be.false;
  });
});
