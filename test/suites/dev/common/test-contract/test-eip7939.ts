import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";

// EIP-7939: CLZ (Count Leading Zeros) opcode
// CLZ returns the number of leading zero bits in a 256-bit value
// Returns 256 (0x100) for input 0

// Runtime bytecode that wraps the CLZ opcode:
// PUSH1 0x00 (60 00) - push offset 0
// CALLDATALOAD (35) - load 32 bytes from calldata
// CLZ (1e) - count leading zeros (EIP-7939)
// PUSH1 0x00 (60 00) - push memory offset 0
// MSTORE (52) - store result to memory
// PUSH1 0x20 (60 20) - push return size 32
// PUSH1 0x00 (60 00) - push return offset 0
// RETURN (f3) - return 32 bytes
const RUNTIME_BYTECODE = "6000351e60005260206000f3";

// Init bytecode that deploys the runtime:
// PUSH1 0x0c (60 0c) - runtime length (12 bytes)
// DUP1 (80) - duplicate length
// PUSH1 0x0b (60 0b) - runtime offset in code (11 bytes = init length)
// PUSH1 0x00 (60 00) - memory offset 0
// CODECOPY (39) - copy runtime to memory
// PUSH1 0x00 (60 00) - return offset 0
// RETURN (f3) - return runtime bytecode
const INIT_BYTECODE = "600c80600b6000396000f3";

const DEPLOY_BYTECODE = "0x" + INIT_BYTECODE + RUNTIME_BYTECODE;

// Test vectors from EIP-7939 specification
// Format: [input, expected_output]
const TEST_VECTORS: [string, bigint][] = [
  // Zero: all 256 bits are zero, so CLZ returns 256 (0x100)
  ["0x0000000000000000000000000000000000000000000000000000000000000000", 256n],
  // Most significant bit set: no leading zeros
  ["0x8000000000000000000000000000000000000000000000000000000000000000", 0n],
  // All bits set: no leading zeros
  ["0xffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 0n],
  // Second most significant bit set: one leading zero
  ["0x4000000000000000000000000000000000000000000000000000000000000000", 1n],
  // All bits set except MSB: one leading zero
  ["0x7fffffffffffffffffffffffffffffffffffffffffffffffffffffffffffffff", 1n],
  // Only least significant bit set: 255 leading zeros
  ["0x0000000000000000000000000000000000000000000000000000000000000001", 255n],
];

describeSuite({
  id: "D010202",
  title: "EIP-7939 - CLZ (Count Leading Zeros) opcode",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let clzContractAddress: `0x${string}`;

    it({
      id: "T01",
      title: "should deploy CLZ test contract",
      test: async function () {
        const hash = await context.viem().sendTransaction({
          data: DEPLOY_BYTECODE as `0x${string}`,
        });

        await context.createBlock();

        const receipt = await context.viem("public").getTransactionReceipt({ hash });

        expect(receipt.status).toBe("success");
        expect(receipt.contractAddress).toBeTruthy();
        clzContractAddress = receipt.contractAddress!;
      },
    });

    it({
      id: "T02",
      title: "should return 256 for zero input",
      test: async function () {
        const result = await context.viem("public").call({
          to: clzContractAddress,
          data: TEST_VECTORS[0][0] as `0x${string}`,
        });

        expect(BigInt(result.data!)).toBe(TEST_VECTORS[0][1]);
      },
    });

    it({
      id: "T03",
      title: "should return 0 for MSB set (0x8000...)",
      test: async function () {
        const result = await context.viem("public").call({
          to: clzContractAddress,
          data: TEST_VECTORS[1][0] as `0x${string}`,
        });

        expect(BigInt(result.data!)).toBe(TEST_VECTORS[1][1]);
      },
    });

    it({
      id: "T04",
      title: "should return 0 for all bits set (0xffff...)",
      test: async function () {
        const result = await context.viem("public").call({
          to: clzContractAddress,
          data: TEST_VECTORS[2][0] as `0x${string}`,
        });

        expect(BigInt(result.data!)).toBe(TEST_VECTORS[2][1]);
      },
    });

    it({
      id: "T05",
      title: "should return 1 for second MSB set (0x4000...)",
      test: async function () {
        const result = await context.viem("public").call({
          to: clzContractAddress,
          data: TEST_VECTORS[3][0] as `0x${string}`,
        });

        expect(BigInt(result.data!)).toBe(TEST_VECTORS[3][1]);
      },
    });

    it({
      id: "T06",
      title: "should return 1 for all except MSB set (0x7fff...)",
      test: async function () {
        const result = await context.viem("public").call({
          to: clzContractAddress,
          data: TEST_VECTORS[4][0] as `0x${string}`,
        });

        expect(BigInt(result.data!)).toBe(TEST_VECTORS[4][1]);
      },
    });

    it({
      id: "T07",
      title: "should return 255 for only LSB set (0x...0001)",
      test: async function () {
        const result = await context.viem("public").call({
          to: clzContractAddress,
          data: TEST_VECTORS[5][0] as `0x${string}`,
        });

        expect(BigInt(result.data!)).toBe(TEST_VECTORS[5][1]);
      },
    });
  },
});
