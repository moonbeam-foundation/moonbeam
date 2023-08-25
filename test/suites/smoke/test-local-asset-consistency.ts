import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { u128 } from "@polkadot/types-codec";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { StorageKey } from "@polkadot/types";

describeSuite({
  id: "S1300",
  title: "Verify local asset count, assetIds and deposits",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise">;
    let localAssetDeposits: StorageKey<[u128]>[];
    let localAssetInfo: StorageKey<[u128]>[];
    let localAssetCounter: number = 0;

    beforeAll(async function () {
      // Configure the api at a specific block
      // (to avoid inconsistency querying over multiple block when the test takes a long time to
      // query data and blocks are being produced)
      atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await context.polkadotJs("para").rpc.chain.getHeader()).number.toNumber();
      apiAt = await context
        .polkadotJs("para")
        .at(await context.polkadotJs("para").rpc.chain.getBlockHash(atBlockNumber));
      localAssetDeposits = await apiAt.query.assetManager.localAssetDeposit.keys();
      localAssetCounter = await (await apiAt.query.assetManager.localAssetCounter()).toNumber();
      localAssetInfo = await apiAt.query.assetManager.localAssetDeposit.keys();
    });

    it({
      id: "C100",
      title: "should match asset deposit entries with number of assets",
      test: async function () {
        expect(
          localAssetDeposits.length,
          `Number of local asset deposits does not much number of local assets`
        ).to.be.eq(localAssetInfo.length);

        log(
          `Verified number of deposits and local asset number matches: ${localAssetDeposits.length}`
        );
      },
    });

    it({
      id: "C200",
      title: "should ensure localAssetCounter is higher than number of local assets",
      test: async function () {
        expect(
          localAssetCounter,
          `Local asset counter lower than total local assets`
        ).to.be.greaterThanOrEqual(localAssetInfo.length);

        log(
          `Verified local asset counter (${localAssetCounter}) 
        >= total local assets: (${localAssetInfo.length})`
        );
      },
    });

    it({
      id: "C300",
      title: `assetIds stored by assetManager are also created in LocalAssets`,
      test: async function () {
        const failedLocalAssets: { assetId: string }[] = [];
        const registeredLocalAssetDeposits = localAssetDeposits.map((set) =>
          set.toHex().slice(-32)
        );
        const registeredLocalAssetInfos = localAssetInfo.map((set) => set.toHex().slice(-32));

        for (const assetId of registeredLocalAssetDeposits) {
          if (!registeredLocalAssetInfos.includes(assetId)) {
            failedLocalAssets.push({ assetId: assetId });
          }
        }

        expect(
          failedLocalAssets.length,
          `Failed deposits with non-existent local assets: ${failedLocalAssets
            .map(({ assetId }) => `expected: ${assetId} to be present in localAssets `)
            .join(`, `)}`
        ).to.equal(0);
        log(
          `Verified ${
            Object.keys(localAssetInfo).length
          } total local assetIds (at #${atBlockNumber})`
        );
      },
    });
  },
});
