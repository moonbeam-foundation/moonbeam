import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import {
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  CONTRACT_RANDOMNESS_STATUS_READY,
  GLMR,
  alith,
} from "@moonwall/util";
import { expectEVMResult, extractRevertReason, SIMPLE_SALT, jumpBlocks } from "../../../../helpers";

describeSuite({
  id: "D013119",
  title: "Randomness VRF - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should refuse a request with less than 2 blocks",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, 1],
          gas: 100_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;
        expectEVMResult(result!.events, "Revert");

        const revertReason = await extractRevertReason(context, result!.hash);
        // Full error expected:
        // Error in pallet_randomness: Module(ModuleError { index: 39, error: [5, 0, 0, 0],
        // message: Some("CannotRequestRandomnessBeforeMinDelay") })
        expect(revertReason).to.contain("CannotRequestRandomnessBeforeMinDelay");

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "should refuse a request with more than 2000 blocks",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            2001, // future blocks
          ],
          gas: 100_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;
        expectEVMResult(result!.events, "Revert");

        const revertReason = await extractRevertReason(context, result!.hash);
        // Full error expected:
        // Error in pallet_randomness: Module(ModuleError { index: 39, error: [4, 0, 0, 0],
        // message: Some("CannotRequestRandomnessAfterMaxDelay") })
        expect(revertReason).to.contain("CannotRequestRandomnessAfterMaxDelay");

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });

    it({
      id: "T03",
      title: "should refuse a request with less than 1 random number",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 0, 2],
          gas: 100_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;
        expectEVMResult(result!.events, "Revert");
        const revertReason = await extractRevertReason(context, result!.hash);
        // Full error expected:
        // Error in pallet_randomness: Module(ModuleError { index: 39, error: [2, 0, 0, 0],
        // message: Some("MustRequestAtLeastOneWord") })
        expect(revertReason).to.contain("MustRequestAtLeastOneWord");

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });

    it({
      id: "T04",
      title: "should refuse a request with more than 100 random number",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 101, 2],
          gas: 100_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;
        expectEVMResult(result!.events, "Revert");
        const revertReason = await extractRevertReason(context, result!.hash);
        // Full error expected:
        // Error in pallet_randomness: Module(ModuleError { index: 39, error: [3, 0, 0, 0],
        // message: Some("CannotRequestMoreWordsThanMax") })
        expect(revertReason).to.contain("CannotRequestMoreWordsThanMax");

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });

    it({
      id: "T05",
      title: "should succeed for 100 random words",
      test: async function () {
        const requestsBefore = (await context.polkadotJs().query.randomness.requests.entries())
          .length;
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 100, 2],
          gas: 100_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length - requestsBefore).to.equal(1);
      },
    });

    it({
      id: "T06",
      title: "should be marked as pending before the end of the delay",
      test: async function () {
        const uid = await context.polkadotJs().query.randomness.requestCount();
        const delayBlocks = 4;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],
          gas: 100_000n,
          rawTxOnly: true,
        });

        await context.createBlock(rawTxn);
        await jumpBlocks(context, delayBlocks - 1);

        expect(
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [uid],
          })
        ).to.equal(CONTRACT_RANDOMNESS_STATUS_PENDING);
      },
    });

    it({
      id: "T07",
      title: "should be marked as ready after delay has passed",
      test: async function () {
        const uid = await context.polkadotJs().query.randomness.requestCount();
        const delayBlocks = 3;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, delayBlocks],
          gas: 100_000n,
          rawTxOnly: true,
        });

        await context.createBlock(rawTxn);
        await jumpBlocks(context, delayBlocks);

        expect(
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [uid],
          })
        ).to.equal(CONTRACT_RANDOMNESS_STATUS_READY);
      },
    });

    it({
      id: "T08",
      title: "should be marked as does not exists",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [1337],
          })
        ).to.equal(CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS);
      },
    });
  },
});
