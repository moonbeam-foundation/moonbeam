import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { fromBytes } from "viem";
import {
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
  verifyLatestBlockFees,
  expectEVMResult,
  registerOldForeignAsset,
  registerXcmTransactorAndContract,
} from "../../../../helpers";

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

describeSuite({
  id: "D012894",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await registerOldForeignAsset(context, RELAY_SOURCE_LOCATION, relayAssetMetadata as any);
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to issue transfer signed xcm transactor with currency Id",
      test: async function () {
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
        // And we need relay tokens for issuing a transaction to be executed in the relay
        const dest: [number, object[]] = [1, []];
        // Destination as currency Id address
        const asset = ADDRESS_RELAY_ASSETS;
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
