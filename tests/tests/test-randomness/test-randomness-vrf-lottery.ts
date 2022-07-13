import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { Contract } from "web3-eth-contract";

import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import {
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  CONTRACT_RANDOMNESS_STATUS_READY,
  PRECOMPILE_RANDOMNESS_ADDRESS,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";

const LOTTERY_CONTRACT = getCompiled("RandomnessLotteryDemo");
const LOTTERY_INTERFACE = new ethers.utils.Interface(LOTTERY_CONTRACT.contract.abi);
const RANDOMNESS_CONTRACT_JSON = getCompiled("Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT_JSON.contract.abi);

const setupLotteryWithParticipants = async (context: DevTestContext) => {
  const { contract, rawTx } = await createContract(context, "RandomnessLotteryDemo");
  await context.createBlock(rawTx);

  // Adds participants
  for (const [privateKey, from] of [
    [ALITH_PRIVATE_KEY, ALITH_ADDRESS],
    [BALTATHAR_PRIVATE_KEY, BALTATHAR_ADDRESS],
    [CHARLETH_PRIVATE_KEY, CHARLETH_ADDRESS],
  ]) {
    await context.createBlock(
      createTransaction(context, {
        ...TRANSACTION_TEMPLATE,
        privateKey,
        from,
        to: contract.options.address,
        data: LOTTERY_INTERFACE.encodeFunctionData("participate", []),
        value: Web3.utils.toWei("1", "ether"),
      })
    );
  }
  return contract;
};

describeDevMoonbeam("Randomness VRF - Lottery Demo", (context) => {
  let lotteryContract: Contract;
  before("setup lottery contract", async function () {
    lotteryContract = await setupLotteryWithParticipants(context);
  });

  it("should be able to start", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: LOTTERY_INTERFACE.encodeFunctionData("startLottery", []),
        value: Web3.utils.toWei("1", "ether"),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });
});

describeDevMoonbeam("Randomness VRF - Lottery Demo", (context) => {
  let lotteryContract: Contract;
  before("setup lottery contract", async function () {
    lotteryContract = await setupLotteryWithParticipants(context);
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: LOTTERY_INTERFACE.encodeFunctionData("startLottery", []),
        value: Web3.utils.toWei("1", "ether"),
      })
    );
  });

  it("should fail to fulfill before the delay", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_PENDING.toString()
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });
});

describeDevMoonbeam("Randomness VRF - Lottery Demo", (context) => {
  let lotteryContract: Contract;
  before("setup lottery contract", async function () {
    lotteryContract = await setupLotteryWithParticipants(context);
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: LOTTERY_INTERFACE.encodeFunctionData("startLottery", []),
        value: Web3.utils.toWei("1", "ether"),
      })
    );
  });

  it("should succeed to fulfill after the delay", async function () {
    await context.createBlock();
    await context.createBlock();
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_READY.toString()
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });
});
