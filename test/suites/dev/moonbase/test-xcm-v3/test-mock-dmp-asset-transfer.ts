import "@moonbeam-network/api-augment";
import { alith, beforeAll, describeSuite, expect } from "moonwall";
import {
  RELAY_SOURCE_LOCATION,
  registerForeignAsset,
  foreignAssetBalance,
  addAssetToWeightTrader,
  relayAssetMetadata,
  injectDownwardMessageAndSeal,
} from "../../../../helpers";

// Twelve decimal places in the moonbase relay chain's token
const RELAY_TOKEN = 1_000_000_000_000n;

describeSuite({
  id: "D024001",
  title: "Mock XCM - receive downward transfer",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const assetId = 1n;

    beforeAll(async () => {
      await registerForeignAsset(context, assetId, RELAY_SOURCE_LOCATION, relayAssetMetadata);

      await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 0n, context);
    });

    it({
      id: "T01",
      title: "Should receive a downward transfer of 10 DOTs to Alith",
      test: async function () {
        // Inject the XCM message and seal until the message queue actually
        // processes it. You can provide a message, but if you don't a downward
        // transfer is the default.
        //
        // Sealing a fixed number of blocks here was racy: the downward message
        // is processed in the message queue's on_idle hook only if enough block
        // weight remains, otherwise it is deferred to a later block.
        await injectDownwardMessageAndSeal(context);

        // Make sure the state has ALITH's to DOT tokens
        const alith_dot_balance = await foreignAssetBalance(
          context,
          assetId,
          alith.address as `0x${string}`
        );

        expect(alith_dot_balance).to.eq(10n * RELAY_TOKEN);
      },
    });
  },
});
