import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { fromBytes } from "viem";
import {
  mockOldAssetBalance,
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorAndContract,
} from "../../../../helpers";

const ADDRESS_RELAY_ASSETS = "0xffffffff1fcacbd218edc0eba20fc2308c778080";

describeSuite({
  id: "D022880",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to issue transfer xcm transactor with currency Id",
      test: async function () {
        // We need to mint units with sudo.setStorage, as we dont have xcm mocker yet
        // And we need relay tokens for issuing a transaction to be executed in the relay

        const balance = context.polkadotJs().createType("Balance", 100000000000000);
        const assetBalance: PalletAssetsAssetAccount = context
          .polkadotJs()
          .createType("PalletAssetsAssetAccount", {
            balance: balance,
          });

        const assetId = context
          .polkadotJs()
          .createType("u128", 42259045809535163221576417993425387648n);
        const assetDetails: PalletAssetsAssetDetails = context
          .polkadotJs()
          .createType("PalletAssetsAssetDetails", {
            supply: balance,
          });

        await mockOldAssetBalance(
          context,
          assetBalance,
          assetDetails,
          alith,
          assetId,
          ALITH_ADDRESS,
          true
        );

        const beforeAssetBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

        const beforeAssetDetails = await context.polkadotJs().query.assets.asset(assetId.toU8a());

        expect(
          beforeAssetBalance.unwrap().balance.toBigInt(),
          "supply and balance should be the same"
        ).to.equal(100000000000000n);
        expect(
          beforeAssetDetails.unwrap().supply.toBigInt(),
          "supply and balance should be the same"
        ).to.equal(100000000000000n);

        const transactor = 0;
        const index = 0;
        // Destination as currency Id address
        const asset = ADDRESS_RELAY_ASSETS;
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const weight = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV1",
          functionName: "transactThroughDerivative",
          args: [transactor, index, asset, weight, transact_call],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        // We have used 1000 units to pay for the fees in the relay, so balance and supply should
        // have changed
        const afterAssetBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

        const expectedBalance = 100000000000000n - 1000n - 1n;
        expect(afterAssetBalance.unwrap().balance.toBigInt()).to.equal(expectedBalance);

        const AfterAssetDetails = await context.polkadotJs().query.assets.asset(assetId.toU8a());

        expect(AfterAssetDetails.unwrap().supply.toBigInt()).to.equal(expectedBalance);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
