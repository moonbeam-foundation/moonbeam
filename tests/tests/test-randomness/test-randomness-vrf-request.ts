import "@moonbeam-network/api-augment/moonbase";
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
import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

const RANDOMNESS_CONTRACT_JSON = getCompiled("precompiles/randomness/Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT_JSON.contract.abi);

const SIMPLE_SALT = new Uint8Array([..."my_salt".padEnd(32, " ")].map((a) => a.charCodeAt(0)));

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should be successful", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          2, // future blocks
        ]),
      })
    );

    expect(result.successful).to.be.true;

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(1);
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  before("setup the request", async function () {
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          2, // future blocks
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

  it("should be considered a local vrf type", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.info.isLocal).to.be.true;
  });

  it("should have a fulfillment block of 3", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.info.asLocal[0].toBigInt()).to.be.equal(3n);
  });

  it("should have an expiration block of 10001", async function () {
    const request = (
      (await context.polkadotApi.query.randomness.requests.entries()) as any
    )[0][1].unwrap().request;
    expect(request.info.asLocal[1].toBigInt()).to.be.equal(10001n);
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should refuse a request with less than 2 blocks", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          1, // future blocks
        ]),
      })
    );

    expect(result.successful).to.be.true;
    expectEVMResult(result.events, "Revert");

    const revertReason = await extractRevertReason(result.hash, context.ethers);
    // Full error expected:
    // Error in pallet_randomness: Module(ModuleError { index: 39, error: [5, 0, 0, 0],
    // message: Some("CannotRequestRandomnessBeforeMinDelay") })
    expect(revertReason).to.contain("CannotRequestRandomnessBeforeMinDelay");

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });

  it("should refuse a request with more than 2000 blocks", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          2001, // future blocks
        ]),
      })
    );

    expect(result.successful).to.be.true;
    expectEVMResult(result.events, "Revert");

    const revertReason = await extractRevertReason(result.hash, context.ethers);
    // Full error expected:
    // Error in pallet_randomness: Module(ModuleError { index: 39, error: [4, 0, 0, 0],
    // message: Some("CannotRequestRandomnessAfterMaxDelay") })
    expect(revertReason).to.contain("CannotRequestRandomnessAfterMaxDelay");

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should refuse a request with less than 1 random number", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          0, // number of random words
          2, // future blocks
        ]),
      })
    );

    expect(result.successful).to.be.true;
    expectEVMResult(result.events, "Revert");
    const revertReason = await extractRevertReason(result.hash, context.ethers);
    // Full error expected:
    // Error in pallet_randomness: Module(ModuleError { index: 39, error: [2, 0, 0, 0],
    // message: Some("MustRequestAtLeastOneWord") })
    expect(revertReason).to.contain("MustRequestAtLeastOneWord");

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should refuse a request with more than 100 random number", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          101, // number of random words
          2, // future blocks
        ]),
      })
    );

    expect(result.successful).to.be.true;
    expectEVMResult(result.events, "Revert");
    const revertReason = await extractRevertReason(result.hash, context.ethers);
    // Full error expected:
    // Error in pallet_randomness: Module(ModuleError { index: 39, error: [3, 0, 0, 0],
    // message: Some("CannotRequestMoreWordsThanMax") })
    expect(revertReason).to.contain("CannotRequestMoreWordsThanMax");

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should succeed for 100 random words", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          100, // number of random words
          2, // future blocks
        ]),
      })
    );

    expect(result.successful).to.be.true;

    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(1);
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should be marked as pending before the end of the delay", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    const delayBlocks = 4;

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          delayBlocks, // future blocks
        ]),
      })
    );

    for (let i = 0; i < delayBlocks - 1; i++) {
      await context.createBlock();
    }

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_PENDING.toString()
    );
  });
});

describeDevMoonbeam("Randomness VRF - Requesting a random number", (context) => {
  it("should be marked as ready after delay has passed", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    const delayBlocks = 3;

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          delayBlocks, // future blocks
        ]),
      })
    );

    for (let i = 0; i < delayBlocks; i++) {
      await context.createBlock();
    }

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_READY.toString()
    );
  });
});

describeDevMoonbeam("Randomness VRF - Requesting an invalid random number", (context) => {
  it("should be marked as does not exists", async function () {
    const randomnessContract = new context.web3.eth.Contract(
      RANDOMNESS_CONTRACT_JSON.contract.abi,
      PRECOMPILE_RANDOMNESS_ADDRESS
    );

    expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
      CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS.toString()
    );
  });
});

describeDevMoonbeam("Randomness VRF - Fulfilling a random request", (context) => {
  before("setup the request", async function () {
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
          alith.address, // refund address
          1n * GLMR, // fee
          100_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
          2, // future blocks
        ]),
      })
    );
    await context.createBlock();
    await context.createBlock();

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );
  });

  it("should remove the request", async function () {
    const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
    expect(randomnessRequests.length).to.equal(0);
  });

  it("should remove the randomness results", async function () {
    const randomnessResults =
      await context.polkadotApi.query.randomness.randomnessResults.entries();
    expect(randomnessResults.length).to.equal(0);
  });
});

describeDevMoonbeam(
  "Randomness VRF - Requesting 2 random requests at same block/delay",
  (context) => {
    before("setup the request", async function () {
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            3, // future blocks
          ]),
        })
      );
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            2, // future blocks
          ]),
        })
      );
      // printEvents(context.polkadotApi);
    });

    it("should create 2 requests", async function () {
      const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
      // console.log(randomnessRequests);
      expect(randomnessRequests.length).to.equal(2);
    });

    it("should have 1 random result", async function () {
      const randomnessResults =
        await context.polkadotApi.query.randomness.randomnessResults.entries();
      expect(randomnessResults.length).to.equal(1);
    });
  }
);

describeDevMoonbeam(
  "Randomness VRF - Fulfilling one of the 2 random requests at same block/delay",
  (context) => {
    before("setup the request", async function () {
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            3, // future blocks
          ]),
        })
      );
      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            2, // future blocks
          ]),
        })
      );
      await context.createBlock();
      await context.createBlock();

      await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_RANDOMNESS_ADDRESS,
          data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
        })
      );
    });

    it("should keep the 2nd request", async function () {
      const randomnessRequests = await context.polkadotApi.query.randomness.requests.entries();
      expect(randomnessRequests.length).to.equal(1);
    });

    it("should keep the randomness results", async function () {
      const randomnessResults =
        await context.polkadotApi.query.randomness.randomnessResults.entries();
      expect(randomnessResults.length).to.equal(1);
    });
  }
);
