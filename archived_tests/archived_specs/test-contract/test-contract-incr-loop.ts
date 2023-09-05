import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { verifyLatestBlockFees } from "../../util/block";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Contract loop creation", (context) => {
  it("Should be initialized at 0", async () => {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await context.createBlock(rawTx);

    expect(await contract.methods.count().call()).to.eq("0");
  });
});

describeDevMoonbeamAllEthTxTypes("Contract loop increment", (context) => {
  it("should increment contract state", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await context.createBlock(rawTx);
    await context.createBlock(
      createContractExecution(context, {
        contract,
        contractCall: contract.methods.incr(),
      })
    );

    expect(await contract.methods.count().call()).to.eq("1");
  });
});

describeDevMoonbeamAllEthTxTypes("Contract loop increment - check fees", (context) => {
  it("should increment contract state", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await context.createBlock(rawTx);
    await context.createBlock(
      createContractExecution(context, {
        contract,
        contractCall: contract.methods.incr(),
      })
    );
    await verifyLatestBlockFees(context);
  });
});
