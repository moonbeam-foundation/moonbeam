import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { ALITH_PRIVATE_KEY } from "@moonwall/util";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
  PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
} from "../../../../helpers";

describeSuite({
  id: "D012800",
  title: "Precompiles - xcm transactor V3",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to transact signed multilocation with custom weights V2 and fee",
      test: async function () {
        const dest: [number, any[]] = [1, []];
        const asset: [number, any[]] = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = { refTime: 1000, proofSize: 1000 };
        const overallWeight = { refTime: 2000, proofSize: 2000 };
        const feeAmount = 1000;
        const refund = true;

        const rawTx = await context.writeContract!({
          contractAddress: PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
          contractName: "XcmTransactorV3",
          functionName: "transactThroughSignedMultilocation",
          args: [dest, asset, transactWeight, transact_call, feeAmount, overallWeight, refund],
          gas: 500_000n,
          rawTxOnly: true,
          privateKey: ALITH_PRIVATE_KEY,
        });

        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
