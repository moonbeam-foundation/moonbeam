import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

describeDevMoonbeam("Pending Pool - Adding", (context) => {
  let txHash;
  before("Setup: Sending a transaction", async function () {
    const { rawTx } = await createContract(context.web3, "TestContract");
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
  });

  it("should return pending transaction by hash", async function () {
    const pendingTransaction = (
      await customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash])
    ).result;
    // pending transactions do not know yet to which block they belong to
    expect(pendingTransaction).to.include({
      blockNumber: null,
      hash: txHash,
      publicKey:
        "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529f" +
        "a2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
      r: "0x64142adbcc090fb188be10d5ce008791f4fb8850b1d364360bd9f8ec2e2f06b8",
      s: "0x141d31f93724fdc78da17fbc47bca20770e558afda2fb75d9de4e4f111c8aeeb",
      v: "0xa25",
    });
  });
});

describeDevMoonbeam("Pending Pool - Creating block", (context) => {
  let txHash;
  before("Setup: Sending a transaction in a block", async function () {
    const { rawTx } = await createContract(context.web3, "TestContract");
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
    await context.createBlock();
  });

  it("should provide pending transactions with block", async function () {
    const processedTransaction = (
      await customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash])
    ).result;
    expect(processedTransaction).to.include({
      blockNumber: "0x1",
      hash: txHash,
      publicKey:
        "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529f" +
        "a2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
      r: "0x64142adbcc090fb188be10d5ce008791f4fb8850b1d364360bd9f8ec2e2f06b8",
      s: "0x141d31f93724fdc78da17fbc47bca20770e558afda2fb75d9de4e4f111c8aeeb",
      v: "0xa25",
    });
  });
});
