import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";
import { registerOldForeignAsset } from "../../../../helpers/assets.js";

const FOREIGN_TOKEN = 1_000_000_000_000n;

const palletId = "0x6D6f646c617373746d6E67720000000000000000";
const statemint_para_id = 1001;
const statemint_assets_pallet_instance = 50;

const assetMetadata = {
  name: "FOREIGN",
  symbol: "FOREIGN",
  decimals: 12n,
  isFrozen: false,
};
const STATEMINT_LOCATION = {
  Xcm: {
    parents: 1,
    interior: {
      X3: [
        { Parachain: statemint_para_id },
        { PalletInstance: statemint_assets_pallet_instance },
        { GeneralIndex: 0 },
      ],
    },
  },
};

describeSuite({
  id: "D024209",
  title: "Mock XCM - receive horizontal transfer",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: string;

    beforeAll(async () => {
      // registerOldForeignAsset
      const { registeredAssetId, registeredAsset } = await registerOldForeignAsset(
        context,
        STATEMINT_LOCATION,
        assetMetadata
      );
      assetId = registeredAssetId;
      expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());
    });

    it({
      id: "T01",
      title: "Should receive a 10 Statemine tokens to Alith with new prefix",
      test: async function () {
        // We are going to test that, using the prefix after
        // https://github.com/paritytech/cumulus/pull/831
        // we can receive the tokens on the assetId registed with the old prefix

        // New prefix:
        // Parachain(Statemint parachain)
        // PalletInstance(Statemint assets pallet instance)
        // GeneralIndex(assetId being transferred)
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 1,
                interior: {
                  X3: [
                    { Parachain: statemint_para_id },
                    { PalletInstance: statemint_assets_pallet_instance },
                    { GeneralIndex: 0n },
                  ],
                },
              },
              fungible: 10000000000000n,
            },
          ],
          weight_limit: {
            refTime: 40000000000n,
            proofSize: 110000n,
          },
          beneficiary: alith.address,
        })
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v5();

        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, statemint_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Make sure the state has ALITH's foreign parachain tokens
        expect(
          (await context.polkadotJs().query.assets.account(assetId, alith.address))
            .unwrap()
            .balance.toBigInt()
        ).to.eq(10n * FOREIGN_TOKEN);
      },
    });
  },
});
