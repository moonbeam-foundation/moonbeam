import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { expectEVMResult, extractRevertReason } from "../../../../helpers";

describeSuite({
  id: "D012989",
  title: "Test GMP Killswitch",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should fail with killswitch enabled by default",
      test: async function () {
        // payload should be irrelevant since the precompile will fail before attempting to decode
        const transferVAA = "deadbeef";

        const rawTxn = await context.writePrecompile!({
          precompileName: "Gmp",
          functionName: "wormholeTransferERC20",
          args: [transferVAA],
          gas: 500_000n,
          rawTxOnly: true,
        });
        const result = await context.createBlock(rawTxn);

        expectEVMResult(result.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(context, result.result!.hash);
        expect(revertReason).to.contain("GMP Precompile is not enabled");
      },
    });
  },
});
