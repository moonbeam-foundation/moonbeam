import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite } from "@moonwall/cli";
import { ALITH_PRIVATE_KEY } from "@moonwall/util";
import { fromBytes } from "viem";
import {
  verifyLatestBlockFees,
  expectEVMResult,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
  registerForeignAsset,
  registerXcmTransactorAndContract,
  PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
} from "../../../../helpers";

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

describeSuite({
  id: "D012991",
  title: "Precompiles - xcm transactor V3",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerForeignAsset(context, RELAY_SOURCE_LOCATION, relayAssetMetadata as any);
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to transact signed with custom weights V2 and fee",
      test: async function () {
        const dest: [number, any[]] = [1, []];
        const asset = ADDRESS_RELAY_ASSETS;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = { refTime: 1000, proofSize: 1000 };
        const overallWeight = { refTime: 2000, proofSize: 2000 };
        const feeAmount = 1000;
        const refund = true;

        const rawTx = await context.writeContract!({
          contractAddress: PRECOMPILE_XCM_TRANSACTOR_V3_ADDRESS,
          contractName: "XcmTransactorV3",
          functionName: "transactThroughSigned",
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
