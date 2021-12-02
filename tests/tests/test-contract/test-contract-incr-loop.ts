import { expect } from "chai";
import { verifyLatestBlockFees } from "../../util/block";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeam("Contract loop creation", (context) => {
  it("Should be initialized at 0", async () => {
    const { contract, rawTx } = await createContract(context.web3, "TestContractIncr");
    await context.createBlock({ transactions: [rawTx] });

    expect(await contract.methods.count().call()).to.eq("0");
  });
});

describeDevMoonbeam("Contract loop increment", (context) => {
  it("should increment contract state", async function () {
    const { contract, rawTx } = await createContract(context.web3, "TestContractIncr");
    await context.createBlock({ transactions: [rawTx] });

    await context.createBlock({
      transactions: [
        await createContractExecution(context.web3, {
          contract,
          contractCall: contract.methods.incr(),
        }),
      ],
    });

    expect(await contract.methods.count().call()).to.eq("1");
  });
});

describeDevMoonbeam("Contract loop increment - check fees", (context) => {
  it("should increment contract state", async function () {
    const { contract, rawTx } = await createContract(context.web3, "TestContractIncr");
    await context.createBlock({ transactions: [rawTx] });

    await context.createBlock({
      transactions: [
        await createContractExecution(context.web3, {
          contract,
          contractCall: contract.methods.incr(),
        }),
      ],
    });
    await verifyLatestBlockFees(context.polkadotApi, expect);
  });
});
