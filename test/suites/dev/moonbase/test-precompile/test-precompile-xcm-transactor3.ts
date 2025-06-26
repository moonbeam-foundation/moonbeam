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

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

describeSuite({
  id: "D022881",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let assetAddress;

    beforeAll(async () => {
      let { contractAddress } = await registerForeignAsset(
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
      title: "allows to issue transfer signed xcm transactor with currency Id",
      test: async function () {
        const dest: [number, object[]] = [1, []];
        // Destination as currency Id address
        // const asset = ADDRESS_RELAY_ASSETS;
        const asset = assetAddress;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const weight = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV1",
          functionName: "transactThroughSigned",
          args: [dest, asset, weight, transact_call],
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
