import "@moonbeam-network/api-augment/moonbase";
import {
  DevModeContext,
  beforeAll,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  DEFAULT_GENESIS_BALANCE,
  GLMR,
  MILLIGLMR,
  alith,
  baltathar,
  charleth,
} from "@moonwall/util";
import { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { nToHex } from "@polkadot/util";
import { TransactionReceipt, decodeEventLog, parseEther } from "viem";

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
  id: "D2702",
  title: "Randomness Babe - Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryAddress: `0x${string}`;
    let fulFillReceipt: TransactionReceipt;

    beforeAll(async function () {
      lotteryAddress = await setupLotteryWithParticipants(context);

      await context.writeContract!({
        contractName: "RandomnessLotteryDemo",
        contractAddress: lotteryAddress,
        functionName: "startLottery",
        gas: 500_000n,
        value: 1n * GLMR,
      });
      await context.createBlock();

      const rawTxn = await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "fulfillRequest",
        args: [0],
        rawTxOnly: true,
        gas: 500_000n,
        privateKey: BALTATHAR_PRIVATE_KEY,
      });

      const { result } = await context.createBlock([fakeBabeResultTransaction(context), rawTxn]);
      fulFillReceipt = await context
        .viem()
        .getTransactionReceipt({ hash: result![1].hash as `0x${string}` });
    });

    it({
      id: "T01",
      title: "should have 4 events",
      test: async function () {
        const decoded = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[0].data,
          topics: fulFillReceipt.logs[0].topics,
        }) as any;

        expect(decoded.eventName).to.equal("Ended");
        expect(decoded.args.participantCount).to.equal(3n);
        expect(decoded.args.jackpot).to.equal(3n * GLMR);
        expect(decoded.args.winnerCount).to.equal(2n);
      },
    });

    it({
      id: "T02",
      title: "should emit 2 Awarded events. One for each winner",
      test: async function () {
        const event2 = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[1].data,
          topics: fulFillReceipt.logs[1].topics,
        }) as any;

        // First Awarded event is for Baltathar
        expect(event2.eventName).to.equal("Awarded");
        expect(event2.args.winner).to.equal(baltathar.address);
        expect(event2.args.randomWord).to.equal(
          74982528826324570542201803903857750688652696143277700801627425400829433687166n
        );
        expect(event2.args.amount).to.equal(1500n * MILLIGLMR);

        // Second Awarded event is for Alith
        const event3 = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[2].data,
          topics: fulFillReceipt.logs[2].topics,
        }) as any;

        expect(event3.eventName).to.equal("Awarded");
        expect(event3.args.winner).to.equal(alith.address);
        expect(event3.args.randomWord).to.equal(
          77024926561716546406163866328460318332430017365028170366735726122254037052683n
        );
        expect(event3.args.amount).to.equal(1500n * MILLIGLMR);
      },
    });

    it({
      id: "T03",
      title: "should emit the FulFillmentSucceeded event last",
      test: async function () {
        const event4 = decodeEventLog({
          abi: fetchCompiledContract("Randomness").abi,
          data: fulFillReceipt.logs[3].data,
          topics: fulFillReceipt.logs[3].topics,
        }) as any;

        expect(event4.eventName).to.equal("FulFillmentSucceeded");
      },
    });

    it({
      id: "T04",
      title: "should remove the request",
      test: async function () {
        expect(
          // await randomnessContract.methods.getRequestStatus(0).call()
          await context.readPrecompile!({
            precompileName: "Randomness",
            functionName: "getRequestStatus",
            args: [0],
          })
        ).to.equal(CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS);

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });

    it({
      id: "T05",
      title: "should reset the jackpot",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryAddress,
            functionName: "jackpot",
          })
        ).to.equal(0n);
      },
    });

    it({
      id: "T06",
      title: "should reward balthazar and alith",
      test: async function () {
        expect(
          (
            await context.polkadotJs().query.system.account(baltathar.address.toString())
          ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
        ).to.be.true;
        expect(
          (
            await context.polkadotJs().query.system.account(charleth.address.toString())
          ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
        ).to.be.false;
        expect(
          (
            await context.polkadotJs().query.system.account(alith.address.toString())
          ).data.free.toBigInt() > ALITH_GENESIS_FREE_BALANCE
        ).to.be.true;
      },
    });

    it({
      id: "T07",
      title: "should be back to open for registrations",
      test: async function () {
        expect(
          await context.readContract!({
            contractName: "RandomnessLotteryDemo",
            contractAddress: lotteryAddress,
            functionName: "status",
          })
        ).to.equal(0);
      },
    });
  },
});
