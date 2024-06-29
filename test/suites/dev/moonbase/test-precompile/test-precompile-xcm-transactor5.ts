import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
  getLastSentUmpMessageFee,
} from "../../../../helpers";

describeSuite({
  id: "D012896",
  title: "Precompiles - xcm transactor V2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const baseDelivery: bigint = 100_000_000_000_000n;
    const txByteFee = 100n;
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to transact signed multilocation with custom weight and fee",
      test: async function () {
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
        // And we need relay tokens for issuing a transaction to be executed in the relay
        const dest: [number, any[]] = [1, []];
        const asset: [number, any[]] = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = 1000;
        const overallWeight = 2000;
        const feeAmount = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV2",
          functionName: "transactThroughSignedMultilocation",
          args: [dest, asset, transactWeight, transact_call, feeAmount, overallWeight],
          gas: 500_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const xcmDeliveryFees = await getLastSentUmpMessageFee(context, baseDelivery, txByteFee);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context, 0n, xcmDeliveryFees);
      },
    });
  },
});
