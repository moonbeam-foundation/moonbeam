import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { TransactionReceipt } from "web3-core";
import { Contract } from "web3-eth-contract";
import {
  alith,
  ALITH_ADDRESS,
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_PRIVATE_KEY,
  baltathar,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import {
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  DEFAULT_GENESIS_BALANCE,
  GLMR,
  MILLIGLMR,
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

const LOTTERY_CONTRACT_JSON = getCompiled("RandomnessLotteryDemo");
const LOTTERY_INTERFACE = new ethers.utils.Interface(LOTTERY_CONTRACT_JSON.contract.abi);
const RANDOMNESS_CONTRACT_JSON = getCompiled("Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT_JSON.contract.abi);

const RANDOMNESS_SOURCE_LOCAL_VRF = "0";
const RANDOMNESS_SOURCE_BABE_EPOCH = "1";

const setupLotteryWithParticipants = async (context: DevTestContext) => {
  const { contract, rawTx } = await createContract(
    context,
    "RandomnessLotteryDemo",
    {
      ...ALITH_TRANSACTION_TEMPLATE,
      value: Web3.utils.toWei("1", "ether"),
      gas: 5_000_000,
    },
    [RANDOMNESS_SOURCE_LOCAL_VRF]
  );
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

describeDevMoonbeam("Randomness VRF - Preparing Lottery Demo", (context) => {
  let lotteryContract: Contract;
  before("setup lottery contract", async function () {
    lotteryContract = await setupLotteryWithParticipants(context);
  });

  it("should have a jackpot of 3 tokens", async function () {
    expect(await lotteryContract.methods.jackpot().call()).to.equal((3n * GLMR).toString());
  });

  it("should be open for registrations", async function () {
    expect(await lotteryContract.methods.status().call()).to.equal("0");
  });
});

describeDevMoonbeam("Randomness VRF - Starting the Lottery Demo", (context) => {
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
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );
    expectEVMResult(result.events, "Error");
  });

  it("should be rolling the numbers", async function () {
    expect(await lotteryContract.methods.status().call()).to.equal("1");
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

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });
});

describeDevMoonbeam("Randomness VRF - Fulfilling Lottery Demo", (context) => {
  let lotteryContract: Contract;
  let randomnessContract: Contract;
  let fulFillReceipt: TransactionReceipt;
  before("setup lottery contract", async function () {
    lotteryContract = await setupLotteryWithParticipants(context);
    randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: lotteryContract.options.address,
        data: LOTTERY_INTERFACE.encodeFunctionData("startLottery", []),
        value: Web3.utils.toWei("1", "ether"),
      })
    );
    await context.createBlock();
    await context.createBlock();
    const {
      result: { hash },
    } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );

    fulFillReceipt = await context.web3.eth.getTransactionReceipt(hash);
  });
  it("should have 4 events", async function () {
    expect(fulFillReceipt.logs.length).to.equal(4);
  });

  it("should emit the Ended log first", async function () {
    const log = LOTTERY_INTERFACE.parseLog(fulFillReceipt.logs[0]);
    expect(log.name).to.equal("Ended");
    expect(log.args.participantCount.toBigInt()).to.equal(3n);
    expect(log.args.jackpot.toBigInt()).to.equal(3n * GLMR);
    expect(log.args.winnerCount.toBigInt()).to.equal(2n);
  });

  it("should emit 2 Awarded events. One for each winner", async function () {
    // First Awarded event is for Charleth
    const log1 = LOTTERY_INTERFACE.parseLog(fulFillReceipt.logs[1]);
    expect(log1.name).to.equal("Awarded");
    expect(log1.args.winner).to.equal(charleth.address);
    expect(log1.args.randomWord.toHexString()).to.equal(
      "0xefb5d3fd7f0afcbebf6c983d4e480100c71395f721e2f3bfdf1c281938947d28"
    );
    expect(log1.args.amount.toBigInt()).to.equal(1500n * MILLIGLMR);

    // Second Awarded event is for Alith
    const log2 = LOTTERY_INTERFACE.parseLog(fulFillReceipt.logs[2]);
    expect(log2.name).to.equal("Awarded");
    expect(log2.args.winner).to.equal(alith.address);
    expect(log2.args.randomWord.toHexString()).to.equal(
      "0xe89db6687fdfcd8523439fa5384e889e028e8fcc1de0ead9f1ba50d5a5aecff8"
    );
    expect(log2.args.amount.toBigInt()).to.equal(1500n * MILLIGLMR);
  });

  it("should emit the FulFillmentSucceeded event last", async function () {
    const log = RANDOMNESS_INTERFACE.parseLog(fulFillReceipt.logs[3]);
    expect(log.name).to.equal("FulFillmentSucceeded");
  });

  it("should remove the request", async function () {
    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS.toString()
    );

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });

  it("should reset the jackpot", async function () {
    expect(await lotteryContract.methods.jackpot().call()).to.equal("0");
  });

  it("should reward alith and charleth", async function () {
    expect(
      (
        await context.polkadotApi.query.system.account(baltathar.address.toString())
      ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
    ).to.be.false;
    expect(
      (
        await context.polkadotApi.query.system.account(charleth.address.toString())
      ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
    ).to.be.true;
    expect(
      (
        await context.polkadotApi.query.system.account(alith.address.toString())
      ).data.free.toBigInt() > ALITH_GENESIS_FREE_BALANCE
    ).to.be.true;
  });

  it("should be back to open for registrations", async function () {
    expect(await lotteryContract.methods.status().call()).to.equal("0");
  });
});
