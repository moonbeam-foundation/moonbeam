import "@moonbeam-network/api-augment";

import { expectEVMResult } from "../../util/eth-transactions";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeam("Precompiles - bn128add", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { contract, rawTx } = await createContract(context, "HasherChecker");
    await context.createBlock(rawTx);

    // Execute the contract bn128add call
    const { result } = await context.createBlock(
      createContractExecution(context, {
        contract,
        contractCall: contract.methods.bn128AdditionCheck(),
      })
    );

    // Verify the result
    expectEVMResult(result.events, "Succeed");
  });
});
