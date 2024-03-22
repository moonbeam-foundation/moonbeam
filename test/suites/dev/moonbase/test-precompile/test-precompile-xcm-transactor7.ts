import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { fromBytes } from "viem";
import {
  mockAssetBalance,
  verifyLatestBlockFees,
  expectEVMResult,
  registerXcmTransactorDerivativeIndex,
} from "../../../../helpers";

describeSuite({
  id: "D012898",
  title: "Precompiles - xcm transactor V2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorDerivativeIndex(context);
      expect(
        await context.readPrecompile!({
          precompileName: "XcmTransactorV2",
          functionName: "indexToAccount",
          args: [0],
        })
      ).toBe(ALITH_ADDRESS);
    });

    it({
      id: "T01",
      title: "allows to transact through derivative multiloc custom fee and weight",
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

        await mockAssetBalance(
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
        const asset: [number, any[]] = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const transactWeight = 500;

        const overallWeight = 1000;
        const feeAmount = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV2",
          functionName: "transactThroughDerivativeMultilocation",
          args: [transactor, index, asset, transactWeight, transact_call, feeAmount, overallWeight],
          gas: 500_000n,
          rawTxOnly: true,
        });

        const result = await context.createBlock(rawTxn);

        expectEVMResult(result.result!.events, "Succeed");

        // We have used 1000 units to pay for the fees in the relay, so balance and supply should
        // have changed
        const afterAssetBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

        const expectedBalance = 100000000000000n - 1000n;
        expect(afterAssetBalance.unwrap().balance.toBigInt()).to.equal(expectedBalance);

        const AfterAssetDetails = await context.polkadotJs().query.assets.asset(assetId.toU8a());

        expect(AfterAssetDetails.unwrap().supply.toBigInt()).to.equal(expectedBalance);

        // 1000 fee for the relay is paid with relay assets
        await verifyLatestBlockFees(context);
      },
    });
  },
});
