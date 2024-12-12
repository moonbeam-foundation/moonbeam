import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createRawTransfer, GLMR, sendRawTransaction } from "@moonwall/util";
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
      title: "should allow resubmitting with higher gas (implying higher tip)",
      test: async function () {
        await context.createBlock([
          await createRawTransfer(context, randomAddress, 1, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("300"),
            maxPriorityFeePerGas: parseGwei("300"),
          }),
          await createRawTransfer(context, randomAddress, 2, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("400"),
            maxPriorityFeePerGas: parseGwei("300"),
            // same priority fee but higher max fee so higher tip
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
      title: "should allow cancelling transaction by reducing limit",
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

        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(1n);
      },
    });

    it({
      id: "T04",
      title: "should prioritize higher gas tips",
      test: async function () {
        // GasFee are using very high value to ensure gasPrice is not impacting
        const alithGLMR = (await context.viem().getBalance({ address: ALITH_ADDRESS })) / GLMR;
        await sendRawTransaction(
          context,
          await createRawTransfer(context, randomAddress, 66, {
            nonce: currentNonce,
            maxFeePerGas: 1n * GLMR,
            maxPriorityFeePerGas: 1n * GLMR,
          })
        );

        const testParameters = [1n * GLMR, 2n * GLMR, 20n * GLMR, 4n * GLMR, 10n * GLMR];
        const txns: string[] = await Promise.all(
          testParameters.map(
            async (gasPrice) =>
              await createRawTransfer(context, randomAddress, 77, {
                nonce: currentNonce,
                maxFeePerGas: gasPrice,
                maxPriorityFeePerGas: gasPrice,
              })
          )
        );

        await context.createBlock(txns);

        expect((await context.viem().getBalance({ address: ALITH_ADDRESS })) / GLMR).to.equal(
          alithGLMR - 21000n * 20n
        );
        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(77n);
      },
    });

    it({
      id: "T05",
      title: "should not allow resubmitting with higher gas (implying same tip)",
      test: async function () {
        await context.createBlock([
          await createRawTransfer(context, randomAddress, 1, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("300"),
            maxPriorityFeePerGas: parseGwei("10"),
          }),
          await createRawTransfer(context, randomAddress, 2, {
            nonce: currentNonce,
            maxFeePerGas: parseGwei("400"),
            maxPriorityFeePerGas: parseGwei("10"),
          }),
        ]);
        expect(await context.viem().getBalance({ address: randomAddress })).to.equal(1n);
      },
    });
  },
});
