import { expect } from "chai";

import { TransactionReceipt } from "web3-core";

import { callContractFunctionMS, deployContractManualSeal, describeWithMoonbeam } from "./util";
import {
  TEST_GAS_ESTIMATION_CALLER_BYTECODE,
  TEST_GAS_ESTIMATION_CALLER_ABI,
  TEST_GAS_ESTIMATION_CALLEE_BYTECODE,
  TEST_GAS_ESTIMATION_CALLEE_ABI,
} from "./constants";

describeWithMoonbeam("Moonbeam RPC (Gas Estimation)", `simple-specs.json`, (context) => {
  it("should estimate reasonable gas", async function () {
    // // instantiate contract
    const callerContract = await deployContractManualSeal(
      context.polkadotApi,
      context.web3,
      TEST_GAS_ESTIMATION_CALLER_BYTECODE,
      TEST_GAS_ESTIMATION_CALLER_ABI,
    );

    const calleeContract = await deployContractManualSeal(
      context.polkadotApi,
      context.web3,
      TEST_GAS_ESTIMATION_CALLEE_BYTECODE,
      TEST_GAS_ESTIMATION_CALLEE_ABI,
    );

    /*
    // call the contract (TODO: not relevant)
    await callerContract.methods
      .someAction(calleeContract.options.address, 1)
      .call();
    */

    expect(await callerContract
           .methods
           .someAction(calleeContract.options.address, 1)
           .estimateGas())
         .to.equal(21204);
  });

});
