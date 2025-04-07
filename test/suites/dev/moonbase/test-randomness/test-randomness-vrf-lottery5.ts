import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  DOROTHY_ADDRESS,
  GLMR,
  MILLIGLMR,
} from "@moonwall/util";
import { type TransactionReceipt, decodeEventLog } from "viem";
import { setupLotteryWithParticipants } from "../../../../helpers";

describeSuite({
  id: "D013115",
  title: "Randomness VRF - Fulfilling Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryContract: `0x${string}`;
    let fulFillReceipt: TransactionReceipt;
    let dorothyBefore: bigint;
    let baltatharBefore: bigint;
    let charlethBefore: bigint;

    beforeAll(async function () {
      [dorothyBefore, baltatharBefore, charlethBefore] = await Promise.all([
        context.viem().getBalance({ address: DOROTHY_ADDRESS }),
        context.viem().getBalance({ address: BALTATHAR_ADDRESS }),
        context.viem().getBalance({ address: CHARLETH_ADDRESS }),
      ]);

      lotteryContract = await setupLotteryWithParticipants(context, "VRF");
      await context.writeContract!({
        contractAddress: lotteryContract,
        contractName: "RandomnessLotteryDemo",
        functionName: "startLottery",
        value: 1n * GLMR,
        gas: 300_000n,
      });
      await context.createBlock();
      await context.createBlock();
      await context.createBlock();

      const estimatedGas = await context.viem().estimateContractGas({
        address: "0x0000000000000000000000000000000000000809",
        abi: fetchCompiledContract("Randomness").abi,
        functionName: "fulfillRequest",
        args: [0],
      });

      expect(estimatedGas).toMatchInlineSnapshot(`149713n`);

      const rawTxn = await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "fulfillRequest",
        args: [0],
        gas: estimatedGas,
        rawTxOnly: true,
      });
      const { result } = await context.createBlock(rawTxn);

      fulFillReceipt = await context
        .viem()
        .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

      expect(fulFillReceipt.gasUsed).toMatchInlineSnapshot(`84956n`);
    });
    it({
      id: "T01",
      title: "should have 4 events",
      test: async function () {
        expect(fulFillReceipt.logs.length).to.equal(4);
      },
    });

    it({
      id: "T02",
      title: "should emit the Ended log first",
      test: async function () {
        const log = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[0].data,
          topics: fulFillReceipt.logs[0].topics,
        }) as any;
        expect(log.eventName).to.equal("Ended");
        expect(log.args.participantCount).to.equal(3n);
        expect(log.args.jackpot).to.equal(3n * GLMR);
        expect(log.args.winnerCount).to.equal(2n);
      },
    });

    it({
      id: "T03",
      title: "should emit 2 Awarded events. One for each winner",
      test: async function () {
        // First Awarded event is for Charleth
        const log1 = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[1].data,
          topics: fulFillReceipt.logs[1].topics,
        }) as any;
        expect(log1.eventName).to.equal("Awarded");
        expect(log1.args.winner).to.equal(CHARLETH_ADDRESS);
        expect(log1.args.randomWord).to.equal(
          51280808134023849127519136205010243437709812126880363876705674960571546808336n
        );
        expect(log1.args.amount).to.equal(1500n * MILLIGLMR);

        // Second Awarded event is for Baltathar
        const log2 = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[2].data,
          topics: fulFillReceipt.logs[2].topics,
        }) as any;
        expect(log2.eventName).to.equal("Awarded");
        expect(log2.args.winner).to.equal(BALTATHAR_ADDRESS);
        expect(log2.args.randomWord).to.equal(
          678783957272396545249253726798886852772291908299890430931444896355209850262n
        );
        expect(log2.args.amount).to.equal(1500n * MILLIGLMR);
      },
    });

    it({
      id: "T04",
      title: "should emit the FulFillmentSucceeded event last",
      test: async function () {
        const log = decodeEventLog({
          abi: fetchCompiledContract("Randomness").abi,
          data: fulFillReceipt.logs[3].data,
          topics: fulFillReceipt.logs[3].topics,
        }) as any;
        expect(log.eventName).to.equal("FulFillmentSucceeded");
      },
    });

    it({
      id: "T05",
      title: "should remove the request",
      test: async function () {
        expect(
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
      id: "T06",
      title: "should reset the jackpot",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: lotteryContract,
            contractName: "RandomnessLotteryDemo",
            functionName: "jackpot",
          })
        ).to.equal(0n);
      },
    });

    it({
      id: "T07",
      title: "should reward baltathar and charleth",
      test: async function () {
        expect(await context.viem().getBalance({ address: DOROTHY_ADDRESS })).toBeLessThan(
          dorothyBefore
        );
        expect(await context.viem().getBalance({ address: BALTATHAR_ADDRESS })).toBeGreaterThan(
          baltatharBefore
        );
        expect(await context.viem().getBalance({ address: CHARLETH_ADDRESS })).toBeGreaterThan(
          charlethBefore
        );
      },
    });

    it({
      id: "T08",
      title: "should be back to open for registrations",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: lotteryContract,
            contractName: "RandomnessLotteryDemo",
            functionName: "status",
          })
        ).to.equal(0);
      },
    });
  },
});
