import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith } from "@moonwall/util";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { fromBytes } from "viem";
import {
  mockOldAssetBalance,
  verifyLatestBlockFees,
  registerXcmTransactorAndContract,
} from "../../../../helpers";

describeSuite({
  id: "D012889",
  title: "Precompiles - xcm transactor",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      await registerXcmTransactorAndContract(context);
    });

    it({
      id: "T01",
      title: "allows to retrieve index through precompiles",
      test: async function () {
        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "indexToAccount",
            args: [0],
          })
        ).toBe(ALITH_ADDRESS);
      },
    });

    it({
      id: "T02",
      title: "allows to retrieve transactor info through precompiles old interface",
      test: async function () {
        // Destination as multilocation, one parent
        const asset: [number, any[]] = [1, []];

        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "transactInfo",
            args: [asset],
          })
        ).toEqual([1n, 1000000000000n, 20000000000n]);
      },
    });

    it({
      id: "T03",
      title: "allows to retrieve fee per second through precompiles",
      test: async function () {
        const asset: [number, any[]] = [1, []];

        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "feePerSecond",
            args: [asset],
          })
        ).toBe(1000000000000n);
      },
    });

    it({
      id: "T04",
      title: "allows to retrieve transactor info through precompiles",
      test: async function () {
        const asset: [number, any[]] = [1, []];

        expect(
          await context.readPrecompile!({
            precompileName: "XcmTransactorV1",
            functionName: "transactInfoWithSigned",
            args: [asset],
          })
        ).toStrictEqual([1n, 1n, 20000000000n]);
      },
    });

    it({
      id: "T05",
      title: "allows to issue transfer xcm transactor",
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
        const asset = [1, []];
        const transact_call = fromBytes(new Uint8Array([0x01]), "hex");
        const weight = 1000;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmTransactorV1",
          functionName: "transactThroughDerivativeMultilocation",
          args: [transactor, index, asset, weight, transact_call],
          rawTxOnly: true,
        });

        await context.createBlock(rawTxn);

        // We have used 1000 units to pay for the fees in the relay  (plus 1 transact_extra_weight),
        // so balance and supply should have changed
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
