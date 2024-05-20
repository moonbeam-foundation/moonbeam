import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S12",
  title: `Verifying foreign asset count, mapping, assetIds and deposits`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise">;
    const foreignAssetIdType: { [assetId: string]: string } = {};
    const foreignAssetTypeId: { [assetType: string]: string } = {};
    const foreignXcmAcceptedAssets: string[] = [];
    let liveForeignAssets: { [key: string]: boolean };
    let specVersion: number;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      // Configure the api at a specific block
      // (to avoid inconsistency querying over multiple block when the test takes a long time to
      // query data and blocks are being produced)
      atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await paraApi.rpc.chain.getHeader()).number.toNumber();

      apiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
      specVersion = apiAt.consts.system.version.specVersion.toNumber();

      let query = await apiAt.query.assetManager.assetIdType.entries();
      query.forEach(([key, exposure]) => {
        const assetId = key.args.toString();
        foreignAssetIdType[assetId] = exposure.unwrap().toString();
      });
      query = await apiAt.query.assetManager.assetTypeId.entries();
      query.forEach(([key, exposure]) => {
        const assetType = key.args.toString();
        foreignAssetTypeId[assetType] = exposure.unwrap().toString();
      });

      query = await apiAt.query.assetManager.assetTypeUnitsPerSecond.entries();

      query.forEach(([key, _]) => {
        const assetType = key.args.toString();
        foreignXcmAcceptedAssets.push(assetType);
      });

      if (specVersion >= 2200) {
        liveForeignAssets = (await apiAt.query.assets.asset.entries()).reduce(
          (acc, [key, value]) => {
            acc[key.args.toString()] = (value.unwrap() as any).status.isLive;
            return acc;
          },
          {} as any
        );
      }
    });

    it({
      id: "C100",
      title: `should make sure xcm fee assets accepted is <=> than existing assets`,
      test: async function () {
        expect(
          foreignXcmAcceptedAssets.length,
          `Number of foreign asset deposits does not match the number of foreign assets`
        ).to.be.lessThanOrEqual(Object.keys(foreignAssetIdType).length);

        log(
          `Verified FOREIGN asset counter (${
            Object.keys(foreignAssetIdType).length
          }) >= xcm fee payment assets: (${foreignXcmAcceptedAssets.length})`
        );
      },
    });

    it({
      id: "C200",
      title: `should make sure assetId->AssetType matches AssetType->AssetId mapping`,
      test: async function () {
        const failedAssetReserveMappings: { assetId: string }[] = [];

        for (const assetId of Object.keys(foreignAssetIdType)) {
          const assetType = foreignAssetIdType[assetId];
          if (foreignAssetTypeId[assetType] != assetId) {
            failedAssetReserveMappings.push({ assetId: assetId });
          }
        }

        expect(
          failedAssetReserveMappings.length,
          `Failed foreign asset entries: ${failedAssetReserveMappings
            .map(({ assetId }) => `expected: ${assetId} to be present in foreignAssets `)
            .join(`, `)}`
        ).to.equal(0);
        log(
          `Verified ${
            Object.keys(foreignAssetIdType).length
          } assetId<->AssetType entries (at #${atBlockNumber})`
        );
      },
    });

    it({
      id: "C300",
      title: `should make sure existing xcm payment assets exist in mapping`,
      test: async function () {
        const failedXcmPaymentAssets: { assetType: string }[] = [];

        for (const assetType of foreignXcmAcceptedAssets) {
          if (!Object.keys(foreignAssetTypeId).includes(assetType)) {
            failedXcmPaymentAssets.push({ assetType });
          }
        }

        expect(
          failedXcmPaymentAssets.length,
          `Failed xcm fee assets: ${failedXcmPaymentAssets
            .map(({ assetType }) => `expected: ${assetType} to be present in foreignAssets `)
            .join(`\n`)}`
        ).to.equal(0);
        log(
          `Verified ${foreignXcmAcceptedAssets.length} xcm ` +
            `fee payment assets (at #${atBlockNumber})`
        );
      },
    });

    it({
      id: "C400",
      title: "should make sure managed assets have live status",
      test: async function () {
        if (specVersion < 2200) {
          log(`ChainSpec ${specVersion} unsupported, skipping.`);
          return;
        }

        const notLiveAssets: string[] = [];
        const assetManagerAssets = Object.keys(foreignAssetIdType);
        for (const assetId of assetManagerAssets) {
          if (!(assetId in liveForeignAssets)) {
            notLiveAssets.push(assetId);
          }
        }

        expect(
          notLiveAssets.length,
          `Failed not live assets - ${notLiveAssets
            .map((assetId) => `expected: ${assetId} to have be a "live" asset`)
            .join(`\n`)}`
        ).to.equal(0);
        log(`Verified ${assetManagerAssets.length} foreign assets (at #${atBlockNumber})`);
      },
    });

    it({
      id: "C500",
      title: "should make sure all live assets are managed",
      test: async function () {
        if (specVersion < 2200) {
          log(`ChainSpec ${specVersion} unsupported, skipping.`);
          return;
        }

        const notLiveAssets: string[] = [];
        const liveAssets = Object.keys(liveForeignAssets);
        for (const assetId of liveAssets) {
          if (!(assetId in foreignAssetIdType)) {
            notLiveAssets.push(assetId);
          }
        }

        expect(
          notLiveAssets.length,
          `Failed not managed live assets - ${notLiveAssets
            .map((assetId) => `expected: ${assetId} to be managed`)
            .join(`\n`)}`
        ).to.equal(0);
        log(`Verified ${liveAssets.length} live assets (at #${atBlockNumber})`);
      },
    });
  },
});
