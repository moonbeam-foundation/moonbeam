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
  it.only("should increment contract state", async function () {
    const { contract, rawTx, contractAddress } = await createContract(context, "TestContractIncr");
    console.log("contractAddress", contractAddress);
    await context.createBlock({ transactions: [rawTx] });
    // console.log(Object.keys(contract));
    // console.log(Object.keys(contract.methods));
    await context.createBlock({
      transactions: [
        await createContractExecution(context, {
          contract,
          contractCall: contract.methods.incr(),
        }),
      ],
    });
    await context.createBlock();
    await context.createBlock();
    await context.createBlock();

    expect(await contract.methods.count().call()).to.eq("1");
  });
});
