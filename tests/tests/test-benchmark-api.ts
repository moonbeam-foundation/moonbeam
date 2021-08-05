import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";

describeDevMoonbeam("Benchmark API", (context) => {
  let fibContract;
  before("Setup: deploy contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "Fibonacci");
    await context.createBlock({ transactions: [rawTx] });

    fibContract = contract;
  });

  it("should return benchmark results", async function () {
    let callTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: fibContract.options.address,
        gas: "0x100000",
        value: "0x00",
        data: fibContract.methods.fib2(1024).encodeABI(),
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    const txn = await customWeb3Request(context.web3, "benchmark_sendRawTransaction", [callTx.rawTransaction]);
    console.log("txn: ", txn);
  });
});
