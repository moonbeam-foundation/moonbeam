import "@moonbeam-network/api-augment/moonbase";
import { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { bnToHex } from "@polkadot/util";
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
  BALTATHAR_TRANSACTION_TEMPLATE,
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
    [RANDOMNESS_SOURCE_BABE_EPOCH]
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

describeDevMoonbeam("Randomness Babe - Preparing Lottery Demo", (context) => {
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

describeDevMoonbeam("Randomness Babe - Starting the Lottery Demo", (context) => {
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

describeDevMoonbeam("Randomness Babe - Lottery Demo", (context) => {
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

  it("should be rolling the numbers", async function () {
    expect(await lotteryContract.methods.status().call()).to.equal("1");
  });
});

describeDevMoonbeam("Randomness Babe - Lottery Demo", (context) => {
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
    // run to beginning of 2nd epoch after epoch in which request was made
    for (let i = 0; i < 15; i++) {
      await context.createBlock();
    }

    const { result } = await context.createBlock([
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      }),
    ]);

    expectEVMResult(result[0].events, "Succeed");
  });
});

describeDevMoonbeam("Randomness Babe - Fulfilling Lottery Demo", (context) => {
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
    // run to beginning of 2nd epoch after epoch in which request was made
    for (let i = 0; i < 15; i++) {
      await context.createBlock();
    }

    const { result } = await context.createBlock([
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE, // mus use baltathar or put correct nonce for alith
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      }),
    ]);
    fulFillReceipt = await context.web3.eth.getTransactionReceipt(result[0].hash);
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
    // First Awarded event is for Baltathar
    const log1 = LOTTERY_INTERFACE.parseLog(fulFillReceipt.logs[1]);
    expect(log1.name).to.equal("Awarded");
    expect(log1.args.winner).to.equal(baltathar.address);
    expect(log1.args.randomWord.toHexString()).to.equal(
      "0x16ded6b78b1f87f393a3a041cb545a0fd460ae968afb44dadcaa4db37979a9fc"
    );
    expect(log1.args.amount.toBigInt()).to.equal(1500n * MILLIGLMR);

    // Second Awarded event is for Alith
    const log2 = LOTTERY_INTERFACE.parseLog(fulFillReceipt.logs[2]);
    expect(log2.name).to.equal("Awarded");
    expect(log2.args.winner).to.equal(alith.address);
    expect(log2.args.randomWord.toHexString()).to.equal(
      "0xef0e7af53cf47bc7309efe730a9a1d5c1e510c1bfbdb6cb80473211e3dc3a2e1"
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

  it("should reward balthazar and alith", async function () {
    expect(
      (
        await context.polkadotApi.query.system.account(baltathar.address.toString())
      ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
    ).to.be.true;
    expect(
      (
        await context.polkadotApi.query.system.account(charleth.address.toString())
      ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
    ).to.be.false;
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
