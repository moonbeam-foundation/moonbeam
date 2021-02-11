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

    // Results from executing this contract in remix:
    //   transaction cost: 66754
    //   execution cost: 43882
    const EXPECTED_GAS = 66754;

    expect(await callerContract
           .methods
           .someAction(calleeContract.options.address, 1)
           .estimateGas())
           .to.equal(EXPECTED_GAS);
  });

});
