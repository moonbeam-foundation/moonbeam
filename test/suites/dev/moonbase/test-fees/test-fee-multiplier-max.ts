import "@moonbeam-network/api-augment/moonbase";
import { beforeEach, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { alith, baltathar, createEthersTransaction } from "@moonwall/util";
import { nToHex } from "@polkadot/util";
import { encodeFunctionData } from "viem";

// Note on the values from 'transactionPayment.nextFeeMultiplier': this storage item is actually a
// FixedU128, which is basically a u128 with an implicit denominator of 10^18. However, this
// denominator is omitted when it is queried through the API, leaving some very large numbers.
//
// To make sense of them, basically remove 18 zeroes (divide by 10^18). This will give you the
// number used internally by transaction-payment for fee calculations.

describeSuite({
  id: "D021502",
  title: "Max Fee Multiplier",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeEach(async () => {
      const MULTIPLIER_STORAGE_KEY = context
        .polkadotJs()
        .query.transactionPayment.nextFeeMultiplier.key(0)
        .toString();

      const U128_MAX = 340282366920938463463374607431768211455n;

      // set transaction-payment's multiplier to something above max in storage. on the next round,
      // it should enforce its upper bound and reset it.
      await context
        .polkadotJs()
        .tx.sudo.sudo(
          context
            .polkadotJs()
            .tx.system.setStorage([
              [MULTIPLIER_STORAGE_KEY, nToHex(U128_MAX, { isLe: true, bitLength: 128 })],
            ])
        )
        .signAndSend(alith);
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should enforce upper bound",
      test: async function () {
        // we set it to u128_max, but the max should have been enforced in on_finalize()
        const multiplier = (
          await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
        ).toBigInt();
        expect(multiplier).toBe(100_000_000_000_000_000_000_000n);
        const gasPrice = await context.viem().getGasPrice();
        expect(gasPrice).toBe(31_250_000_000_000n);
      },
    });

    it({
      id: "T02",
      title: "should have spendable runtime upgrade",
      test: async () => {
        const multiplier = (
          await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
        ).toBigInt();
        expect(multiplier).to.equal(100_000_000_000_000_000_000_000n);

        const initialBalance = (
          await context.polkadotJs().query.system.account(baltathar.address as string)
        ).data.free.toBigInt();

        // generate a mock runtime upgrade hex string
        const size = 4194304; // 2MB bytes represented in hex
        const hex = "0x" + "F".repeat(size);

        // send an applyAuthorizedUpgrade. we expect this to fail, but we just want to see that it
        // was included in a block (not rejected) and was charged based on its length
        await context.polkadotJs().tx.system.applyAuthorizedUpgrade(hex).signAndSend(baltathar);
        await context.createBlock();

        const afterBalance = (
          await context.polkadotJs().query.system.account(baltathar.address as string)
        ).data.free.toBigInt();

        // note that this is not really affected by the high multiplier because most of its fee is
        // derived from the length_fee, which is not scaled by the multiplier
        // ~/4 to compensate for the ref time XCM fee changes
        // Previous value: 449_284_776_265_723_667_008n
        // Previous value: 119_241_298_837_127_813_277n
        // Previous value: 146_015_659_550_552_813_277n
        // Previous value: 146_015_659_564_090_313_277n
        expect(initialBalance - afterBalance).toMatchInlineSnapshot(`231664695814090313277n`);
      },
    });

    it({
      id: "T03",
      title: "should have spendable fill_block",
      test: async () => {
        const multiplier = (
          await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
        ).toBigInt();
        expect(multiplier).to.equal(100_000_000_000_000_000_000_000n);

        // fill_block will not charge its full amount for us, but we can inspect the initial balance
        // withdraw event to see what it would charge. it is root only and will refund if not called
        // by root, but sudo will also cause a refund.

        const fillAmount = 600_000_000; // equal to 60% Perbill

        const { block, result } = await context.createBlock(
          context.polkadotJs().tx.rootTesting.fillBlock(fillAmount),
          { allowFailures: true }
        );

        // grab the first withdraw event and hope it's the right one...
        const withdrawEvent = result?.events.filter(({ event }) => event.method === "Withdraw")[0];
        const amount = withdrawEvent.event.data.amount.toBigInt();
        // ~/4 to compensate for the ref time XCM fee changes
        // Previous value: 6_000_000_012_598_000_941_192n
        expect(amount).to.equal(1_500_000_003_224_000_970_299n);
      },
    });

    // similar to tests in test-contract-fibonacci.ts, which implements an Ethereum txn which uses
    // most of the block gas limit. This is done with the fee at its max, however.
    it({
      id: "T04",
      title: "fibonacci[370] should be spendable",
      test: async function () {
        let blockNumber = (await context.polkadotJs().rpc.chain.getHeader()).number.toBigInt();
        let baseFeePerGas = (await context.viem().getBlock({ blockNumber: blockNumber }))
          .baseFeePerGas!;
        expect(baseFeePerGas).to.equal(31_250_000_000_000n);

        const {
          hash: createTxHash,
          contractAddress,
          abi: contractAbi,
        } = await deployCreateCompiledContract(context, "Fibonacci");

        const receipt = await context.viem().getTransactionReceipt({ hash: createTxHash });
        expect(receipt.status).toBe("success");

        // the multiplier (and thereby base_fee) will have decreased very slightly...
        blockNumber = (await context.polkadotJs().rpc.chain.getHeader()).number.toBigInt();
        baseFeePerGas = (await context.viem().getBlock({ blockNumber: blockNumber }))
          .baseFeePerGas!;
        expect(baseFeePerGas).to.equal(31_206_751_955_281n);

        const rawSigned = await createEthersTransaction(context, {
          to: contractAddress,
          data: encodeFunctionData({
            abi: contractAbi,
            functionName: "fib2",
            args: [370],
          }),
          gasLimit: 95132, // Replace this if OPCODE prices change
          gasPrice: baseFeePerGas,
          txnType: "legacy",
        });
        const { result: interactionResult } = await context.createBlock(rawSigned);

        const receipt2 = await context
          .viem("public")
          .getTransactionReceipt({ hash: interactionResult!.hash as `0x${string}` });
        expect(receipt2.status).toBe("success");

        const successEvent = interactionResult?.events.filter(
          ({ event }) => event.method === "ExtrinsicSuccess"
        )[0];
        const weight = successEvent.event.data.dispatchInfo.weight.refTime.toBigInt();
        expect(weight).to.equal(1_734_300_000n);

        const withdrawEvents = interactionResult?.events.filter(
          ({ event }) => event.method === "Withdraw"
        );
        expect(withdrawEvents?.length).to.equal(1);
        const withdrawEvent = withdrawEvents![0];
        const amount = withdrawEvent.event.data.amount.toBigInt();
        expect(amount).to.equal(2_968_760_727_009_792_092n);
      },
    });
  },
});
