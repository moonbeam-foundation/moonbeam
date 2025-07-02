import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  DEFAULT_GENESIS_BALANCE,
  GLMR,
  MILLIGLMR,
  baltathar,
  charleth,
  dorothy,
} from "@moonwall/util";
import { type TransactionReceipt, decodeEventLog } from "viem";
import {
  fakeBabeResultTransaction,
  setupLotteryWithParticipants,
} from "../../../../helpers/randomness.js";

describeSuite({
  id: "D023102",
  title: "Randomness Babe - Lottery Demo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let lotteryAddress: `0x${string}`;
    let fulFillReceipt: TransactionReceipt;

    beforeAll(async function () {
      lotteryAddress = await setupLotteryWithParticipants(context, "BABE");

      await context.writeContract!({
        contractName: "RandomnessLotteryDemo",
        contractAddress: lotteryAddress,
        functionName: "startLottery",
        gas: 500_000n,
        value: 1n * GLMR,
      });

      await context.createBlock();

      await context.createBlock([fakeBabeResultTransaction(context)]);

      const estimatedGas = await context.viem().estimateContractGas({
        address: "0x0000000000000000000000000000000000000809",
        abi: fetchCompiledContract("Randomness").abi,
        functionName: "fulfillRequest",
        args: [0],
        account: BALTATHAR_ADDRESS,
      });
      expect(estimatedGas).to.equal(151470n);

      const rawTxn = await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "fulfillRequest",
        args: [0],
        gas: estimatedGas,
        rawTxOnly: true,
        privateKey: BALTATHAR_PRIVATE_KEY,
      });

      // We fake the results twice, once to estimate the gas
      // and once to actually fulfill the request
      const { result } = await context.createBlock([fakeBabeResultTransaction(context), rawTxn]);
      fulFillReceipt = await context
        .viem()
        .getTransactionReceipt({ hash: result![1].hash as `0x${string}` });
      expect(fulFillReceipt.gasUsed).to.equal(84110n);
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

        // Second Awarded event is for Dorothy
        const event3 = decodeEventLog({
          abi: fetchCompiledContract("RandomnessLotteryDemo").abi,
          data: fulFillReceipt.logs[2].data,
          topics: fulFillReceipt.logs[2].topics,
        }) as any;

        expect(event3.eventName).to.equal("Awarded");
        expect(event3.args.winner).to.equal(dorothy.address);
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
            await context.polkadotJs().query.system.account(baltathar.address)
          ).data.free.toBigInt() > DEFAULT_GENESIS_BALANCE
        ).to.be.true;
        expect(
          (await context.polkadotJs().query.system.account(charleth.address)).data.free.toBigInt() >
            DEFAULT_GENESIS_BALANCE
        ).to.be.false;
        expect(
          (await context.polkadotJs().query.system.account(dorothy.address)).data.free.toBigInt() >
            DEFAULT_GENESIS_BALANCE
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
