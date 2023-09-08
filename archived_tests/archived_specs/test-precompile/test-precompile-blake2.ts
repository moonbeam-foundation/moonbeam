import "@moonbeam-network/api-augment";

import { expectEVMResult } from "../../util/eth-transactions";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeam("Precompiles - blake2", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { contract, rawTx } = await createContract(context, "HasherChecker");
    await context.createBlock(rawTx);

    // Execute the contract blake2 call
    const { result } = await context.createBlock(
      createContractExecution(context, {
        contract,
        contractCall: contract.methods.blake2Check(),
      })
    );

    // Verify the result
    expectEVMResult(result.events, "Succeed");
  });
});
