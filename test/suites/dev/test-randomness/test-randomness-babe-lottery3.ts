import "@moonbeam-network/api-augment/moonbase";
import {
  DevModeContext,
  beforeAll,
  beforeEach,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  GLMR,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { encodeFunctionData, parseEther } from "viem";
import { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { bnToHex, nToHex } from "@polkadot/util";
const RANDOMNESS_SOURCE_LOCAL_VRF = "0";
const RANDOMNESS_SOURCE_BABE_EPOCH = "1";

const setupLotteryWithParticipants = async (context: DevModeContext) => {
  const { contractAddress: lotteryAddress } = await context.deployContract!(
    "RandomnessLotteryDemo",
    {
      args: [RANDOMNESS_SOURCE_BABE_EPOCH],
      value: parseEther("1"),
      gas: 5_000_000n,
    }
  );

  for (const [privateKey, from] of [
    [ALITH_PRIVATE_KEY, ALITH_ADDRESS],
    [BALTATHAR_PRIVATE_KEY, BALTATHAR_ADDRESS],
    [CHARLETH_PRIVATE_KEY, CHARLETH_ADDRESS],
  ]) {
    await context.writeContract!({
      contractName: "RandomnessLotteryDemo",
      contractAddress: lotteryAddress,
      functionName: "participate",
      args: [],
      gas: 500_000n,
      value: parseEther("1"),
      privateKey,
    });
  }
  await context.createBlock();
  return lotteryAddress;
};

// Uses sudo (alith) to set relayEpoch to +2 and randomnessResult to the desired value
const fakeBabeResultTransaction = async (
  context: DevModeContext,
  value?: PalletRandomnessRandomnessResult
) => {
  const fakeRandomResult = context.polkadotJs().registry.createType(
    "Option<PalletRandomnessRandomnessResult>",
    value || {
      requestCount: 1,
      randomness: "0xb1ffdd4a26e0f2a2fd1e0862a1c9be422c66dddd68257306ed55dc7bd9dce647",
    }
  );

  return context
    .polkadotJs()
    .tx.sudo.sudo(
      context.polkadotJs().tx.system.setStorage([
        [
          context.polkadotJs().query.randomness.relayEpoch.key().toString(),
          nToHex((await context.polkadotJs().query.randomness.relayEpoch()).toBigInt() + 2n, {
            bitLength: 64,
            isLe: true,
          }),
        ],
        [
          context.polkadotJs().query.randomness.randomnessResults.key({ BabeEpoch: 2 }).toString(),
          fakeRandomResult.toHex(),
        ],
      ])
    )
    .signAsync(alith);
};

describeSuite({
  id: "D2703",
  title: "Randomness Babe - Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryAddress: `0x${string}`;

    beforeEach(async function () {
      lotteryAddress = await setupLotteryWithParticipants(context);

      await context.writeContract!({
        contractName: "RandomnessLotteryDemo",
        contractAddress: lotteryAddress,
        functionName: "startLottery",
        gas: 500_000n,
        value: 1n * GLMR,
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should fail to fulfill before the delay",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [0],
          })
        ).toBe(CONTRACT_RANDOMNESS_STATUS_PENDING);

        const rawTxn = await createViemTransaction(context, {
          to: lotteryAddress,
          data: encodeFunctionData({
            abi: fetchCompiledContract("Randomness").abi,
            functionName: "fulfillRequest",
            args: [0],
          }),
          gas: 500_000n,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");

        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryAddress,
            functionName: "status",
          })
        ).to.equal(1);
      },
    });

    it({
      id: "T02",
      title: "should succeed to fulfill after the delay",
      test: async function () {
        await context.createBlock();

        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "fulfillRequest",
          args: [0],
          gas: 500_000n,
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
        });
        const { result } = await context.createBlock([
          // Faking relay epoch + 2 in randomness storage
          fakeBabeResultTransaction(context),
          rawTxn,
        ]);

        expectEVMResult(result![1].events, "Succeed");
      },
    });
  },
});
