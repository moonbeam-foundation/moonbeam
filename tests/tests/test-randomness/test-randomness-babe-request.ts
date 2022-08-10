import "@moonbeam-network/api-augment";
import { u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { alith } from "../../util/accounts";
import {
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  CONTRACT_RANDOMNESS_STATUS_READY,
  GLMR,
  PRECOMPILE_RANDOMNESS_ADDRESS,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

const RANDOMNESS_CONTRACT_JSON = getCompiled("Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT_JSON.contract.abi);

const SIMPLE_SALT = new Uint8Array([..."my_salt".padEnd(32, " ")].map((a) => a.charCodeAt(0)));

describeDevMoonbeam("Randomness Babe - Requesting a random number", (context) => {
  it("should be successful", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
        ]),
      })
    );

    expect(result.successful).to.be.true;

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(1);
  });
});

describeDevMoonbeam("Randomness Babe - Requesting a random number", (context) => {
  before("setup the request", async function () {
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
        ]),
      })
    );
  });

  it("should store a request with id:0", async function () {
    const requestId = parseInt(
      ((await context.polkadotApi.query.randomness.requests.entries()) as any)[0][0]
        .toHex()
        .slice(-16),
      16
    );
    expect(requestId).to.equal(0);
  });

  it("should store the salt", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.salt.toHex()).to.equal(u8aToHex(SIMPLE_SALT));
  });

  it("should store the refundAddress", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.refundAddress.toHex()).to.equal(alith.address.toLocaleLowerCase());
  });

  // This is a bit weird as we are calling the precompile from a non smart-contract
  it("should store the contractAddress", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.contractAddress.toHex()).to.equal(alith.address.toLocaleLowerCase());
  });

  it("should store the fee", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.fee.toBigInt()).to.equal(1n * GLMR);
  });

  it("should store the gasLimit", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.gasLimit.toBigInt()).to.equal(100_000n);
  });

  it("should store the numWords", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.numWords.toBigInt()).to.equal(1n);
  });

  it("should be considered a babe type", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.info.isBabeEpoch).to.be.true;
  });

  it("should have a fulfillment delay of 2 epochs", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.info.asBabeEpoch[0].toBigInt()).to.be.equal(2n);
  });

  it("should have an expiration delay of 10001 epochs", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.info.asBabeEpoch[1].toBigInt()).to.be.equal(10000n);
  });
});

describeDevMoonbeam("Randomness Babe - Requesting a random number", (context) => {
  it("should refuse a request with more than 100 random number", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          101, // number of random words
        ]),
      })
    );

    expect(result.successful).to.be.true;
    expectEVMResult(result.events, "Revert");

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });
});

describeDevMoonbeam("Randomness Babe - Requesting a random number", (context) => {
  it("should succeed for 100 random words", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          100, // number of random words
        ]),
      })
    );

    expect(result.successful).to.be.true;

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(1);
  });
});

describeDevMoonbeam("Randomness Babe - Requesting a random number", (context) => {
  it("should be marked as pending before the end of the 2nd epoch", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
        ]),
      })
    );

    for (let i = 0; i < 10; i++) {
      await context.createBlock();
    }

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_PENDING.toString()
    );
  });
});

describeDevMoonbeam("Randomness Babe - Requesting a random number", (context) => {
  // TODO: Fix it once we support setting the epochs properly
  it.skip("should be marked as ready after 2 epochs has passed", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
        ]),
      })
    );

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_READY.toString()
    );
  });
});

describeDevMoonbeam("Randomness Babe - Requesting an invalid random number", (context) => {
  it("should be marked as pending before the end of the delay", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS.toString()
    );
  });
});
