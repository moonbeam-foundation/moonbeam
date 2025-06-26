import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { fromBytes } from "viem";
import {
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
  registerForeignAsset,
} from "../../../../helpers";

// const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

describeSuite({
  id: "D012897",
  title: "Precompiles - xcm transactor V2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetAddress;

    beforeAll(async () => {
      const { contractAddress } = await registerForeignAsset(
        context,
        1n,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata
      );
      assetAddress = contractAddress;
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to transact signed with custom weight and fee",
      test: async function () {
        const dest: [number, any[]] = [1, []];
        const asset = assetAddress;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = 1000;
        const overallWeight = 2000;
        const feeAmount = 1000;

        // Call the precompile

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV2",
          functionName: "transactThroughSigned",
          args: [dest, asset, transactWeight, transact_call, feeAmount, overallWeight],
          gas: 500_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
