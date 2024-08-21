import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { u128 } from "@polkadot/types";
import { hexToBigInt } from "@polkadot/util";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { generateKeyringPair, alith } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  relayAssetMetadata,
  RELAY_SOURCE_LOCATION,
  registerOldForeignAsset,
  mockOldAssetBalance,
} from "../../../../helpers";

// TODO: remove once we upgrade @polkadot/api to v12.1.1
const runtimeApi = {
  runtime: {
    XcmPaymentApi: [
      {
        methods: {
          query_acceptable_payment_assets: {
            description: "The API to query acceptable payment assets",
            params: [
              {
                name: "version",
                type: "u32",
              },
            ],
            type: "Result<Vec<XcmVersionedAssetId>, XcmPaymentApiError>",
          },
          query_weight_to_asset_fee: {
            description: "",
            params: [
              {
                name: "weight",
                type: "WeightV2",
              },
              {
                name: "asset",
                type: "XcmVersionedAssetId",
              },
            ],
            type: "Result<u128, XcmPaymentApiError>",
          },
          query_xcm_weight: {
            description: "",
            params: [
              {
                name: "message",
                type: "XcmVersionedXcm",
              },
            ],
            type: "Result<WeightV2, XcmPaymentApiError>",
          },
          query_delivery_fees: {
            description: "",
            params: [
              {
                name: "destination",
                type: "XcmVersionedLocation",
              },
              {
                name: "message",
                type: "XcmVersionedXcm",
              },
            ],
            type: "Result<XcmVersionedAssets, XcmPaymentApiError>",
          },
        },
        version: 1,
      },
    ],
  },
  types: {
    XcmPaymentApiError: {
      _enum: {
        Unimplemented: "Null",
        VersionedConversionFailed: "Null",
        WeightNotComputable: "Null",
        UnhandledXcmVersion: "Null",
        AssetNotFound: "Null",
      },
    },
  },
};

describeSuite({
  id: "D014133",
  title: "XCM - XcmPaymentApi - Transact",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let polkadotJs: ApiPromise;
    let amountForFees: bigint;
    let amountForTransfer: bigint;
    let assetId: u128;
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let random: KeyringPair;
    let foreignAssetId: string;
    const weightLimit = {
      refTime: 40_000_000_000n,
      proofSize: 120_000n,
    };
    let weightToForeignFee: any;

    beforeAll(async () => {
      // TODO: remove once we upgrade @polkadot/api to v12.1.1
      polkadotJs = await ApiPromise.create({
        provider: new WsProvider(`ws://localhost:${process.env.MOONWALL_RPC_PORT}/`),
        ...runtimeApi,
      });

      const { registeredAssetId } = await registerOldForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata as any,
        20000000000
      );

      foreignAssetId = registeredAssetId;

      // Fetch the exact amount of foreign fees that we will use given
      // the indicated weightLimit
      weightToForeignFee = await polkadotJs.call.xcmPaymentApi.queryWeightToAssetFee(weightLimit, {
        V3: {
          Concrete: { parents: 1, interior: "Here" },
        },
      });

      expect(weightToForeignFee.isOk).to.be.true;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      descendAddress = descendOriginAddress;
      sendingAddress = originAddress;

      random = generateKeyringPair();
      // Amount to use inside BuyExecution
      amountForFees = BigInt(weightToForeignFee.asOk.toJSON());
      // Amount to transfer to random address
      amountForTransfer = 1_000_000_000_000_000_000n;

      const balance = polkadotJs.createType("Balance", amountForFees);
      assetId = polkadotJs.createType("u128", hexToBigInt(foreignAssetId as `0x${string}`));

      const assetBalance: PalletAssetsAssetAccount = polkadotJs.createType(
        "PalletAssetsAssetAccount",
        {
          balance: balance,
        }
      );
      const assetDetails: PalletAssetsAssetDetails = polkadotJs.createType(
        "PalletAssetsAssetDetails",
        {
          supply: balance,
        }
      );

      // Fund descendAddress with enough xcDOTs to pay XCM execution fees
      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        descendAddress
      );

      // We need to fund the descendAddress with both amounts.
      // This account takes care of paying the foreign fees and also transfering the
      // native tokens to the random address.
      await context.createBlock(
        polkadotJs.tx.balances.transferAllowDeath(descendAddress, amountForTransfer),
        { allowFailures: false }
      );

      const descendForeignBalance = (
        await polkadotJs.query.assets.account(foreignAssetId, descendAddress)
      )
        .unwrap()
        .balance.toBigInt();

      const descendNativeBalance = (
        await polkadotJs.query.system.account(descendAddress)
      ).data.free.toBigInt();
      expect(descendForeignBalance).to.eq(amountForFees);
      expect(descendNativeBalance).to.eq(amountForTransfer);
    });

    it({
      id: "T01",
      title: "Should de able to transact using the estimated foreign fees",
      test: async function () {
        // Build Transact encoded call
        const transferCall = polkadotJs.tx.balances.transferAllowDeath(
          random.address,
          amountForTransfer
        );
        const transferCallEncoded = transferCall?.method.toHex();

        // Build the XCM message with the corresponding weightLimit
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 1,
                interior: { Here: null },
              },
              fungible: amountForFees,
            },
          ],
          weight_limit: weightLimit,
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 1_000_000_000n,
                proofSize: 80_000n,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v4();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the random address received the transfer
        const testAccountBalance = (
          await polkadotJs.query.system.account(random.address)
        ).data.free.toBigInt();

        // Make sure the descendOrigin address has zero foreign balance now
        const testDescendBalance = (
          await polkadotJs.query.assets.account(foreignAssetId, descendAddress)
        )
          .unwrap()
          .balance.toBigInt();

        expect(testAccountBalance).to.eq(amountForTransfer);
        expect(testDescendBalance).to.eq(0n);
      },
    });
  },
});
