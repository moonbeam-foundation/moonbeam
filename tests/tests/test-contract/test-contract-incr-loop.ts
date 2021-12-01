import { expect } from "chai";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Contract loop creation", (context) => {
  it("Should be initialized at 0", async () => {
    const { contract, rawTx } = await createContract(context, "TestContractIncr");
    await context.createBlock({ transactions: [rawTx] });

    expect(await contract.methods.count().call()).to.eq("0");
  });
});

describeDevMoonbeamAllEthTxTypes("Contract loop increment", (context) => {
  it("should increment contract state", async function () {
    const { contract, rawTx, contractAddress } = await createContract(context, "TestContractIncr");
    await context.createBlock({ transactions: [rawTx] });
    await context.createBlock({
      transactions: [
        await createContractExecution(context, {
          contract,
          contractCall: contract.methods.incr(),
        }),
      ],
    });

    expect(await contract.methods.count().call()).to.eq("1");
  });
});
