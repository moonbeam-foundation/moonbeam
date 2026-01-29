import "@moonbeam-network/api-augment";
import { ALITH_ADDRESS, describeSuite, expect, fetchCompiledContract } from "moonwall";

describeSuite({
  id: "D021701",
  title: "Estimate Gas - Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should return contract creation gas cost",
      test: async function () {
        const { bytecode } = fetchCompiledContract("MultiplyBy7");
        expect(
          await context.viem().estimateGas({
            account: ALITH_ADDRESS,
            data: bytecode,
          })
        ).to.equal(210541n);
      },
    });
  },
});
