import "@moonbeam-network/api-augment";
import { deployCreateCompiledContract, describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { hexToU8a } from "@polkadot/util";
import { encodeDeployData, keccak256, numberToHex, toRlp } from "viem";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

// TODO: expand these tests to do multiple txn types when added to viem
describeSuite({
  id: "D4007",
  title: "Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      const { contractAddress, abi } = await deployCreateCompiledContract(
        context,
        "ProxyForContract"
      );
    });

    it({
      id: `T01`,
      title: `should not be able to execute`,
      test: async function () {
        const { hash } = await deployCreateCompiledContract(context, "MultiplyBy7");
        await context.createBlock();
        expect(hash).toBeTruthy();
      },
    });
  },
});
