import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:localAssets");

describeSmokeSuite(
  "S1000",
  `Verifying foreign asset count, mapping, assetIds and deposits`,

  (context, testIt) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;
    const foreignAssetIdType: { [assetId: string]: string } = {};
    const foreignAssetTypeId: { [assetType: string]: string } = {};
    const foreignXcmAcceptedAssets: string[] = [];
    let liveForeignAssets: { [key: string]: boolean };
    let specVersion: number;

    before("Setup api & retrieve data", async function () {
      // Configure the api at a specific block
      // (to avoid inconsistency querying over multiple block when the test takes a long time to
      // query data and blocks are being produced)
      atBlockNumber = process.env.BLOCK_NUMBER
        ? parseInt(process.env.BLOCK_NUMBER)
        : (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
      apiAt = await context.polkadotApi.at(
        await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
      );
      specVersion = apiAt.consts.system.version.specVersion.toNumber();

      let query = await apiAt.query.assetManager.assetIdType.entries();
      query.forEach(([key, exposure]) => {
        let assetId = key.args.toString();
        foreignAssetIdType[assetId] = exposure.unwrap().toString();
      });
      query = await apiAt.query.assetManager.assetTypeId.entries();
      query.forEach(([key, exposure]) => {
        let assetType = key.args.toString();
        foreignAssetTypeId[assetType] = exposure.unwrap().toString();
      });

      query = await apiAt.query.assetManager.assetTypeUnitsPerSecond.entries();
      query.forEach(([key, _]) => {
        let assetType = key.args.toString();
        foreignXcmAcceptedAssets.push(assetType);
      });

      if (specVersion >= 2200) {
        liveForeignAssets = (await apiAt.query.assets.asset.entries()).reduce(
          (acc, [key, value]) => {
            acc[key.args.toString()] = (value.unwrap() as any).status.isLive;
            return acc;
          },
          {}
        );
      }
    });

    testIt(
      "C100",
      `should make sure xcm fee assets accepted is <=> than existing assets`,
      async function () {
        expect(
          foreignXcmAcceptedAssets.length,
          `Number of local asset deposits does not much number of local assets`
        ).to.be.lessThanOrEqual(Object.keys(foreignAssetIdType).length);

        debug(
          `Verified FOREIGN asset counter (${
            Object.keys(foreignAssetIdType).length
          }) >= xcm fee payment assets: (${foreignXcmAcceptedAssets.length})`
        );
      }
    );

    testIt(
      "C200`",
      `should make sure assetId->AssetType matches AssetType->AssetId mapping`,
      async function () {
        const failedAssetReserveMappings: { assetId: string }[] = [];

        for (const assetId of Object.keys(foreignAssetIdType)) {
          let assetType = foreignAssetIdType[assetId];
          if (foreignAssetTypeId[assetType] != assetId) {
            failedAssetReserveMappings.push({ assetId: assetId });
          }
        }

        expect(
          failedAssetReserveMappings.length,
          `Failed foreign asset entries: ${failedAssetReserveMappings
            .map(({ assetId }) => `expected: ${assetId} to be present in localAssets `)
            .join(`, `)}`
        ).to.equal(0);
        debug(
          `Verified ${
            Object.keys(foreignAssetIdType).length
          } assetId<->AssetType entries (at #${atBlockNumber})`
        );
      }
    );

    testIt(
      "C300",
      `should make sure existing xcm payment assets exist in mapping`,
      async function () {
        const failedXcmPaymentAssets: { assetType: string }[] = [];

        for (const assetType of foreignXcmAcceptedAssets) {
          if (!Object.keys(foreignAssetTypeId).includes(assetType)) {
            failedXcmPaymentAssets.push({ assetType });
          }
        }

        expect(
          failedXcmPaymentAssets.length,
          `Failed xcm fee assets: ${failedXcmPaymentAssets
            .map(({ assetType }) => `expected: ${assetType} to be present in localAssets `)
            .join(`\n`)}`
        ).to.equal(0);
        debug(
          `Verified ${foreignXcmAcceptedAssets.length} xcm ` +
            `fee payment assets (at #${atBlockNumber})`
        );
      }
    );

    testIt("C400", `should make sure managed assets have live status`, async function () {
      if (specVersion < 2200) {
        debug(`ChainSpec ${specVersion} unsupported, skipping.`);
        this.skip();
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
      debug(`Verified ${assetManagerAssets.length} foreign assets (at #${atBlockNumber})`);
    });

    testIt("C500", `should make sure all live assets are managed`, async function () {
      if (specVersion < 2200) {
        debug(`ChainSpec ${specVersion} unsupported, skipping.`);
        this.skip();
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
      debug(`Verified ${liveAssets.length} live assets (at #${atBlockNumber})`);
    });
  }
);
