import "@moonbeam-network/api-augment";

import { ethers } from "ethers";

import { alith } from "../../util/accounts";
import { GLMR, PRECOMPILE_RANDOMNESS_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

const RANDOMNESS_CONTRACT = getCompiled("Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT.contract.abi);

describeDevMoonbeam("Precompile Randomness - VRF", (context) => {
  it("should allow to create a randomness request", async function () {
    await context.createBlock();
    const validationData = await context.polkadotApi.query.parachainSystem.validationData();
    const relayParentNumber = validationData.unwrap().relayParentNumber.toNumber();
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalRandomness", [
          alith.address,
          1n * GLMR,
          1_000_000,
          "my_rand",
          relayParentNumber + 1,
        ]),
      })
    );

    expectEVMResult(result.events, "Succeed", "Stopped");
  });
});
