import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";

describeSuite({
  id: "D1801",
  title: "Estimate Gas - Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should return contract creation gas cost",
      test: async function () {
        const { bytecode } = await fetchCompiledContract("MultiplyBy7");
        expect(
          await context.viem("public").estimateGas({
            account: ALITH_ADDRESS,
            data: bytecode,
          })
        ).to.equal(157029n);
      },
    });
  },
});
