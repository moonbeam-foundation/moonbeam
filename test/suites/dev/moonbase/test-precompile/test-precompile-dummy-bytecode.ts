import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  DUMMY_REVERT_BYTECODE,
  PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
  createEthersTransaction,
} from "@moonwall/util";
import * as RLP from "rlp";
import { keccak256 } from "viem";

// push1 5 (deployed bytecode length)
// dup1
// push1 11 (offset of deployed bytecode in this initcode)
// push1 0 (offset in target memory)
// codecopy (copy code slice into memory)
// push1 0 (offset in target memory)
// return
// <deployed bytecode>
const INIT_CODE = "0x600580600B6000396000F360006000fd";

describeSuite({
  id: "D012938",
  title: "Precompiles - precompiles dummy bytecode",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should return dummy bytecode for every precompiles",
      test: async function () {
        const addresses: Array<`0x${string}`> = [
          "0x0000000000000000000000000000000000000001",
          "0x0000000000000000000000000000000000000002",
          "0x0000000000000000000000000000000000000003",
          "0x0000000000000000000000000000000000000004",
          "0x0000000000000000000000000000000000000005",
          "0x0000000000000000000000000000000000000006",
          "0x0000000000000000000000000000000000000007",
          "0x0000000000000000000000000000000000000008",
          "0x0000000000000000000000000000000000000400",
          "0x0000000000000000000000000000000000000401",
          "0x0000000000000000000000000000000000000402",
          PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
        ];

        const matches = await Promise.all(
          addresses.map(async (address) => {
            const code = await context.viem().getBytecode({ address: address as `0x${string}` });
            return code === DUMMY_REVERT_BYTECODE;
          })
        );

        const failures = matches.filter((match) => !match);
        failures.forEach((failure, index) => {
          log(`Failure at address ${addresses[index]}`);
        });

        expect(
          failures.length,
          "Contract calls to precompile doesn't equal DUMMY_REVERT_BYTECODE"
        ).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "should revert when dummy bytecode is called",
      test: async function () {
        // we deploy a new contract with the same bytecode to be able to
        // execute the bytecode instead of executing a precompile.
        await context.createBlock(
          createEthersTransaction(context, {
            data: INIT_CODE,
          })
        );

        const contractAddress =
          "0x" +
          keccak256(RLP.encode([ALITH_ADDRESS, 0]), "hex")
            .slice(12)
            .substring(14);
        // check the deployed code by this init code watches what we use for precompiles.
        const code = await context
          .viem()
          .getBytecode({ address: contractAddress as `0x${string}` });
        expect(code).to.equal(DUMMY_REVERT_BYTECODE);

        // try to call contract (with empty data, shouldn't matter)

        const { result } = await context.createBlock(
          createEthersTransaction(context, {
            gas: 12_000_000,
            data: "0x",
            to: contractAddress,
          })
        );
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

        expect(receipt.status).to.equal("reverted");
        // 21006 = call cost + 2*PUSH cost
        expect(receipt.gasUsed).to.equal(21006n);
      },
    });
  },
});
