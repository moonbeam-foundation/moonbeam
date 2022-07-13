import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { Contract } from "web3-eth-contract";

import {
  alith,
  ALITH_PRIVATE_KEY,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import { jumpToRound } from "../../util/block";
import { GLMR, PRECOMPILE_RANDOMNESS_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createContractExecution,
  createTransaction,
} from "../../util/transactions";
import { deployContract } from "../test-precompile/test-precompile-democracy";

const LOTTERY_CONTRACT = getCompiled("RandomnessLotteryDemo");

describeDevMoonbeam("Randomness VRF - Lottery Demo", (context) => {
  let lotteryContract: Contract;
  before("setup lottery contract", async function () {
    const { contract, rawTx } = await createContract(context, "RandomnessLotteryDemo");
    lotteryContract = contract;
    await context.createBlock(rawTx);

    // Adds participants
    for (const privateKey of [ALITH_PRIVATE_KEY, BALTATHAR_PRIVATE_KEY, CHARLETH_PRIVATE_KEY]) {
      await context.createBlock(
        createContractExecution(
          context,
          {
            contract,
            contractCall: contract.methods.participate(),
          },
          { privateKey, value: Web3.utils.toWei("1", "ether") }
        )
      );
    }
  });

  it("should be successful", async function () {});
});
