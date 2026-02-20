import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "moonwall";
import type { Abi } from "viem";

// EIP-7939: CLZ (Count Leading Zeros) opcode
// CLZ returns the number of leading zero bits in a 256-bit value
// Returns 256 (0x100) for input 0

// Test vectors from EIP-7939 specification
const TEST_VECTORS: [bigint, bigint][] = [
  // Zero: all 256 bits are zero, so CLZ returns 256 (0x100)
  [0n, 256n],
  // Most significant bit set: no leading zeros
  [0x8000000000000000000000000000000000000000000000000000000000000000n, 0n],
  // All bits set: no leading zeros
  [0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn, 0n],
  // Second most significant bit set: one leading zero
  [0x4000000000000000000000000000000000000000000000000000000000000000n, 1n],
  // All bits set except MSB: one leading zero
  [0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffn, 1n],
  // Only least significant bit set: 255 leading zeros
  [1n, 255n],
];

describeSuite({
  id: "D010202",
  title: "EIP-7939 - CLZ (Count Leading Zeros) opcode",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let clzContractAddress: `0x${string}`;
    let clzAbi: Abi;

    beforeAll(async () => {
      // Deploy CLZ contract compiled with solc 0.8.31
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "CLZ");
      expect(contractAddress).toBeTruthy();
      clzContractAddress = contractAddress;
      clzAbi = abi;
    });

    it({
      id: "T01",
      title: "should return 256 for zero input",
      test: async function () {
        const result = await context.viem().readContract({
          address: clzContractAddress,
          abi: clzAbi,
          functionName: "countLeadingZeros",
          args: [TEST_VECTORS[0][0]],
        });

        expect(result).toBe(TEST_VECTORS[0][1]);
      },
    });

    it({
      id: "T02",
      title: "should return 0 for MSB set (0x8000...)",
      test: async function () {
        const result = await context.viem().readContract({
          address: clzContractAddress,
          abi: clzAbi,
          functionName: "countLeadingZeros",
          args: [TEST_VECTORS[1][0]],
        });

        expect(result).toBe(TEST_VECTORS[1][1]);
      },
    });

    it({
      id: "T03",
      title: "should return 0 for all bits set (0xffff...)",
      test: async function () {
        const result = await context.viem().readContract({
          address: clzContractAddress,
          abi: clzAbi,
          functionName: "countLeadingZeros",
          args: [TEST_VECTORS[2][0]],
        });

        expect(result).toBe(TEST_VECTORS[2][1]);
      },
    });

    it({
      id: "T04",
      title: "should return 1 for second MSB set (0x4000...)",
      test: async function () {
        const result = await context.viem().readContract({
          address: clzContractAddress,
          abi: clzAbi,
          functionName: "countLeadingZeros",
          args: [TEST_VECTORS[3][0]],
        });

        expect(result).toBe(TEST_VECTORS[3][1]);
      },
    });

    it({
      id: "T05",
      title: "should return 1 for all except MSB set (0x7fff...)",
      test: async function () {
        const result = await context.viem().readContract({
          address: clzContractAddress,
          abi: clzAbi,
          functionName: "countLeadingZeros",
          args: [TEST_VECTORS[4][0]],
        });

        expect(result).toBe(TEST_VECTORS[4][1]);
      },
    });

    it({
      id: "T06",
      title: "should return 255 for only LSB set (0x...0001)",
      test: async function () {
        const result = await context.viem().readContract({
          address: clzContractAddress,
          abi: clzAbi,
          functionName: "countLeadingZeros",
          args: [TEST_VECTORS[5][0]],
        });

        expect(result).toBe(TEST_VECTORS[5][1]);
      },
    });
  },
});
