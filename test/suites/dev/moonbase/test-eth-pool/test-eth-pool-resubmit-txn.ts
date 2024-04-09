import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createRawTransfer, sendRawTransaction } from "@moonwall/util";
import { parseGwei } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D011105",
  title: "Resubmit transations",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let randomAddress: `0x${string}`;
    let currentNonce: number;

    beforeEach(async function () {
      randomAddress = privateKeyToAccount(generatePrivateKey()).address;
      currentNonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });
    });

    it({
      id: "T01",
      title: "should allow resubmitting with higher gas",
      test: async function () {
        await context.createBlock([
          await createRawTransfer(context, randomAddress, 1, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("10"),
          }),
          await createRawTransfer(context, randomAddress, 2, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("20"),
          }),
        ]);
        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(2n);
      },
    });

    it({
      id: "T02",
      title: "should ignore resubmitting with lower gas",
      test: async function () {
        await context.createBlock([
          await createRawTransfer(context, randomAddress, 1, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("20"),
          }),
          await createRawTransfer(context, randomAddress, 2, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("10"),
          }),
        ]);
        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(1n);
      },
    });

    it({
      id: "T03",
      title: "should allow cancelling transaction",
      test: async function () {
        // gas price should trump limit
        await context.createBlock([
          await createRawTransfer(context, randomAddress, 1, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("10"),
            gas: 1048575n,
          }),
          await createRawTransfer(context, randomAddress, 2, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("20"),
            gas: 65536n,
          }),
        ]);

        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(2n);
      },
    });

    it({
      id: "T04",
      title: "should pick highest gas price from many transactions",
      test: async function () {
        await sendRawTransaction(
          context,
          await createRawTransfer(context, randomAddress, 2, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("100"),
            maxPriorityFeePerGas: parseGwei("100"),
          })
        );

        const testParameters = [
          parseGwei("2"),
          parseGwei("5"),
          parseGwei("10"),
          parseGwei("11"),
          parseGwei("20"),
        ];
        const txns: string[] = await Promise.all(
          testParameters.map(
            async (gasPrice) =>
              await createRawTransfer(context, randomAddress, 1, {
                nonce: currentNonce,
                maxFeePerGas: gasPrice,
                maxPriorityFeePerGas: gasPrice,
              })
          )
        );

        await context.createBlock(txns);

        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(2n);
      },
    });
  },
});
