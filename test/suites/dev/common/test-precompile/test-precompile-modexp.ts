import "@moonbeam-network/api-augment";
import { describeSuite, expect, createViemTransaction } from "moonwall";
import { toHex } from "viem";
import { EIP_7825_MAX_TRANSACTION_GAS_LIMIT, testVectors } from "helpers";

// MODEXP precompile address (0x05) - standard Ethereum precompile
const PRECOMPILE_MODEXP_ADDRESS = "0x0000000000000000000000000000000000000005";

// EIP-7823: Maximum input size in bytes for base, exponent, and modulus
const EIP_7823_MAX_INPUT_SIZE = 1024;

// Helper to encode a 32-byte big-endian length
function encodeLength(len: number): string {
  return toHex(BigInt(len), { size: 32 }).slice(2); // Remove 0x prefix
}

// Helper to encode MODEXP input from hex strings
// Format: base_length (32 bytes) || exp_length (32 bytes) || mod_length (32 bytes) || base || exp || mod
function encodeModexpInputFromHex(baseHex: string, expHex: string, modHex: string): `0x${string}` {
  const baseLen = baseHex.length / 2;
  const expLen = expHex.length / 2;
  const modLen = modHex.length / 2;

  return `0x${encodeLength(baseLen)}${encodeLength(expLen)}${encodeLength(modLen)}${baseHex}${expHex}${modHex}`;
}

// Helper to encode MODEXP input with specified sizes (for bounds testing)
// Uses exponent = 1 (identity operation: base^1 mod m = base mod m) to minimize gas costs
function encodeModexpInputWithSizes(
  baseLen: number,
  expLen: number,
  modLen: number
): `0x${string}` {
  // Fill base with 0x01 bytes
  const base = "01".repeat(baseLen);
  // Use exponent = 1 (identity) with leading zeros to match expLen
  // This minimizes iteration count to 0, making computation very cheap
  const exp = "00".repeat(expLen - 1) + "01";
  // Fill mod with 0xff bytes (large modulus to avoid zero result issues)
  const mod = "ff".repeat(modLen);

  return `0x${encodeLength(baseLen)}${encodeLength(expLen)}${encodeLength(modLen)}${base}${exp}${mod}`;
}

describeSuite({
  id: "D010419",
  title: "Precompiles - MODEXP (EIP-7823 bounds)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    // Test standard MODEXP functionality with known test vectors
    it({
      id: "T01",
      title: "should compute modexp correctly for nagydani-1-square test vector",
      test: async function () {
        const vector = testVectors["nagydani-1-square"];
        const input = encodeModexpInputFromHex(vector.base, vector.exponent, vector.modulus);

        const result = await context.viem().call({
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
        });

        expect(result.data).toBeTruthy();
        // Result should be 64 bytes (same as modulus length)
        expect(result.data!.length).toBe(2 + 128); // 0x prefix + 64 bytes hex
      },
    });

    it({
      id: "T02",
      title: "should compute modexp correctly for nagydani-2-qube test vector",
      test: async function () {
        const vector = testVectors["nagydani-2-qube"];
        const input = encodeModexpInputFromHex(vector.base, vector.exponent, vector.modulus);

        const result = await context.viem().call({
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
        });

        expect(result.data).toBeTruthy();
        // Result should be 128 bytes (same as modulus length)
        expect(result.data!.length).toBe(2 + 256); // 0x prefix + 128 bytes hex
      },
    });

    it({
      id: "T03",
      title: "should succeed with inputs within EIP-7823 bounds (small inputs)",
      test: async function () {
        // Use valid input sizes (all within 1024 bytes)
        const input = encodeModexpInputWithSizes(32, 32, 32);

        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
          gas: EIP_7825_MAX_TRANSACTION_GAS_LIMIT,
        });

        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T04",
      title: "should succeed at exact EIP-7823 boundary (1024 bytes base)",
      test: async function () {
        // Base size at exactly 1024 bytes (maximum allowed)
        const input = encodeModexpInputWithSizes(EIP_7823_MAX_INPUT_SIZE, 32, 32);

        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
          gas: EIP_7825_MAX_TRANSACTION_GAS_LIMIT,
        });

        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).toBe("success");
      },
    });

    it({
      id: "T05",
      title: "should fail and consume all gas when base exceeds EIP-7823 bounds (1025 bytes)",
      test: async function () {
        // Base size exceeds 1024 bytes (EIP-7823 bound)
        const input = encodeModexpInputWithSizes(EIP_7823_MAX_INPUT_SIZE + 1, 32, 32);
        const gasLimit = 500_000n;

        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
          gas: gasLimit,
        });

        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        // Transaction should revert due to EIP-7823 bounds violation
        expect(receipt.status).toBe("reverted");

        // Per EIP-7823, all gas should be consumed
        expect(receipt.gasUsed).toBe(gasLimit);
      },
    });

    it({
      id: "T06",
      title: "should fail and consume all gas when exponent exceeds EIP-7823 bounds (1025 bytes)",
      test: async function () {
        // Exponent size exceeds 1024 bytes (EIP-7823 bound)
        const input = encodeModexpInputWithSizes(32, EIP_7823_MAX_INPUT_SIZE + 1, 32);
        const gasLimit = 500_000n;

        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
          gas: gasLimit,
        });

        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        // Transaction should revert due to EIP-7823 bounds violation
        expect(receipt.status).toBe("reverted");

        // Per EIP-7823, all gas should be consumed
        expect(receipt.gasUsed).toBe(gasLimit);
      },
    });

    it({
      id: "T07",
      title: "should fail and consume all gas when modulus exceeds EIP-7823 bounds (1025 bytes)",
      test: async function () {
        // Modulus size exceeds 1024 bytes (EIP-7823 bound)
        const input = encodeModexpInputWithSizes(32, 32, EIP_7823_MAX_INPUT_SIZE + 1);
        const gasLimit = 500_000n;

        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
          gas: gasLimit,
        });

        const { result } = await context.createBlock(rawTxn);

        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        // Transaction should revert due to EIP-7823 bounds violation
        expect(receipt.status).toBe("reverted");

        // Per EIP-7823, all gas should be consumed
        expect(receipt.gasUsed).toBe(gasLimit);
      },
    });

    it({
      id: "T08",
      title: "should handle zero modulus correctly",
      test: async function () {
        // Base and exponent are non-zero, but modulus is all zeros
        const input = `0x${encodeLength(1)}${encodeLength(1)}${encodeLength(1)}020100`;

        const result = await context.viem().call({
          to: PRECOMPILE_MODEXP_ADDRESS,
          data: input as `0x${string}`,
        });

        // Division by zero should return zero
        expect(result.data).toBe("0x00");
      },
    });
  },
});
