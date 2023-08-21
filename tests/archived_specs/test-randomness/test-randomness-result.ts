import "@moonbeam-network/api-augment/moonbase";
import { Option } from "@polkadot/types";
import { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { expect } from "chai";
import { ethers } from "ethers";
import { alith } from "../../util/accounts";
import { GLMR, PRECOMPILE_RANDOMNESS_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

const RANDOMNESS_CONTRACT_JSON = getCompiled("precompiles/randomness/Randomness");
const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT_JSON.contract.abi);

const SIMPLE_SALT = new Uint8Array([..."my_salt".padEnd(32, " ")].map((a) => a.charCodeAt(0)));

describeDevMoonbeam(
  "Randomness Result - Requesting 4 random numbers for the same target block",
  (context) => {
    it("should only have 1 randomness result with 4 requests", async function () {
      await context.createBlock([
        ...[0, 1].map((nonce) =>
          createTransaction(context, {
            ...ALITH_TRANSACTION_TEMPLATE,
            to: PRECOMPILE_RANDOMNESS_ADDRESS,
            nonce,
            data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
              alith.address, // refund address
              1n * GLMR, // fee
              100_000n, // gas limit
              SIMPLE_SALT,
              1, // number of random words
              3, // future blocks
            ]),
          })
        ),
      ]);
      await context.createBlock([
        ...[2, 3].map((nonce) =>
          createTransaction(context, {
            ...ALITH_TRANSACTION_TEMPLATE,
            to: PRECOMPILE_RANDOMNESS_ADDRESS,
            nonce,
            data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
              alith.address, // refund address
              1n * GLMR, // fee
              100_000n, // gas limit
              SIMPLE_SALT,
              1, // number of random words
              2, // future blocks
            ]),
          })
        ),
      ]);

      const randomessResults =
        await context.polkadotApi.query.randomness.randomnessResults.entries();
      expect(randomessResults).to.be.length(1);
      const randomessResult = randomessResults[0][1] as Option<PalletRandomnessRandomnessResult>;
      expect(randomessResult.unwrap().requestCount.toNumber()).to.equal(4);
      expect(randomessResult.unwrap().randomness.isNone).to.be.true;
    });
  }
);

describeDevMoonbeam("Randomness Result - Fulfilling one of multiple random numbers", (context) => {
  it("should leave 1 randomness result", async function () {
    const delayBlocks = 2;
    const requestData = RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
      alith.address, // refund address
      1n * GLMR, // fee
      100_000n, // gas limit
      SIMPLE_SALT,
      1, // number of random words
      delayBlocks, // future blocks
    ]);
    await context.createBlock([
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: requestData,
      }),
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        nonce: 1,
        data: requestData,
      }),
    ]);

    for (let i = 0; i < delayBlocks; i++) {
      await context.createBlock();
    }

    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      })
    );

    const randomessResults = await context.polkadotApi.query.randomness.randomnessResults.entries();
    expect(randomessResults).to.be.length(1);
    const randomessResult = randomessResults[0][1] as Option<PalletRandomnessRandomnessResult>;
    expect(randomessResult.unwrap().requestCount.toNumber()).to.equal(1);
    expect(randomessResult.unwrap().randomness.isSome).to.be.true;
  });
});

describeDevMoonbeam("Randomness Result - Fulfilling all of random numbers", (context) => {
  it("should empty randomness results", async function () {
    const delayBlocks = 2;
    const requestData = RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
      alith.address, // refund address
      1n * GLMR, // fee
      100_000n, // gas limit
      SIMPLE_SALT,
      1, // number of random words
      delayBlocks, // future blocks
    ]);
    await context.createBlock([
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: requestData,
      }),
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        nonce: 1,
        data: requestData,
      }),
    ]);

    for (let i = 0; i < delayBlocks; i++) {
      await context.createBlock();
    }

    await context.createBlock([
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
      }),

      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_RANDOMNESS_ADDRESS,
        nonce: 3,
        data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [1]),
      }),
    ]);

    const randomessResults = await context.polkadotApi.query.randomness.randomnessResults.entries();
    expect(randomessResults).to.be.length(0);
  });
});

describeDevMoonbeam("Randomness Result - Passing targetted block", (context) => {
  it("should fill the randomness value", async function () {
    const delayBlocks = 2;
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

    const randomessResults = await context.polkadotApi.query.randomness.randomnessResults.entries();
    expect(randomessResults).to.be.length(1);
    const randomessResult = randomessResults[0][1] as Option<PalletRandomnessRandomnessResult>;
    expect(randomessResult.unwrap().randomness.isSome).to.be.true;
    expect(randomessResult.unwrap().randomness.unwrap().toHex()).to.equal(
      "0xb1ffdd4a26e0f2a2fd1e0862a1c9be422c66dddd68257306ed55dc7bd9dce647"
    );
  });
});
