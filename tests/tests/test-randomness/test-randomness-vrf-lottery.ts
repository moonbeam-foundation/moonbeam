import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { Contract } from "web3-eth-contract";

import {
  alith,
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  baltathar,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_ADDRESS,
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
  TRANSACTION_TEMPLATE
} from "../../util/transactions";
import { deployContract } from "../test-precompile/test-precompile-democracy";
import { web3EthCall } from "../../util/providers";

const LOTTERY_CONTRACT = getCompiled("RandomnessLotteryDemo");
const LOTTERY_INTERFACE = new ethers.utils.Interface(LOTTERY_CONTRACT.contract.abi);

describeDevMoonbeam("Randomness VRF - Lottery Demo", (context) => {
  let lotteryContract: Contract;
  before("setup lottery contract", async function () {
    const { contract, rawTx } = await createContract(context, "RandomnessLotteryDemo");
    lotteryContract = contract;
    await context.createBlock(rawTx);

    // Adds participants
    for (const [privateKey, from] of [[ALITH_PRIVATE_KEY, ALITH_ADDRESS], [BALTATHAR_PRIVATE_KEY, BALTATHAR_ADDRESS], [CHARLETH_PRIVATE_KEY, CHARLETH_ADDRESS]]) {
      await context.createBlock(
        createTransaction(context, {
          ...TRANSACTION_TEMPLATE,
          privateKey,
          from,
          to: lotteryContract.options.address,
          data: LOTTERY_INTERFACE.encodeFunctionData("participate", []),
          value:  Web3.utils.toWei("1", "ether")
        })
      );
    }
  });

  it("should be successful", async function () {

    const { result } =  await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: LOTTERY_INTERFACE.encodeFunctionData("startLottery", []),
        value:  Web3.utils.toWei("1", "ether")
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(true);
  });
  
});