import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, getCompiled } from "@moonwall/util";

describeSuite({
  id: "D1701",
  title: "Estimate Gas - Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should return contract creation gas cost",
      test: async function () {
        const contract = getCompiled("MultiplyBy7");
        expect(
          await context.viemClient("public").estimateGas({
            account: ALITH_ADDRESS,
            data: contract.byteCode,
          })
        ).to.equal(156994n);
      },
    });
  },
});
