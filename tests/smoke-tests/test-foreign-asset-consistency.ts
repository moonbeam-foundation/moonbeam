import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:localAssets");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(
  `Verify foreign asset count, mapping, assetIds and deposits`,
  { wssUrl, relayWssUrl },
  (context) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;
    const foreignAssetIdType: { [assetId: string]: string } = {};
    const foreignAssetTypeId: { [assetType: string]: string } = {};
    const foreignXcmAcceptedAssets: string[] = [];

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
    });

    it("should make sure xcm fee assets accepted is <=> than existing assets", async function () {
      expect(
        foreignXcmAcceptedAssets.length,
        `Number of local asset deposits does not much number of local assets`
      ).to.be.lessThanOrEqual(Object.keys(foreignAssetIdType).length);

      debug(
        `Verified FOREIGN asset counter (${
          Object.keys(foreignAssetIdType).length
        }) >= xcm fee payment assets: (${foreignXcmAcceptedAssets.length})`
      );
    });

    it("should make sure assetId->AssetType matches AssetType->AssetId mapping", async function () {
      // Instead of putting an expect in the loop. We track all failed entries instead
      const failedAssetReserveMappings: { assetId: string }[] = [];

      for (const assetId of Object.keys(foreignAssetIdType)) {
        let assetType = foreignAssetIdType[assetId];
        if (foreignAssetTypeId[assetType] != assetId) {
          failedAssetReserveMappings.push({ assetId: assetId });
        }
      }

      console.log("Failed foreign assetType<->AssetId entries:");
      console.log(
        failedAssetReserveMappings
          .map(({ assetId }) => {
            return `expected: ${assetId} to be present in localAssets `;
          })
          .join(`\n`)
      );

      // Make sure the test fails after we print the errors
      expect(failedAssetReserveMappings.length, "Failed local assets").to.equal(0);

      // Additional debug logs
      debug(
        `Verified ${
          Object.keys(foreignAssetIdType).length
        } assetId<->AssetType entries (at #${atBlockNumber})`
      );
    });

    it("should make sure existing xcm payment assets exist in mapping", async function () {
      // Instead of putting an expect in the loop. We track all failed entries instead
      const failedXcmPaymentAssets: { assetType: string }[] = [];

      for (const assetType of foreignXcmAcceptedAssets) {
        if (!Object.keys(foreignAssetTypeId).includes(assetType)) {
          failedXcmPaymentAssets.push({ assetType });
        }
      }

      console.log("Failed xcm fee payment assets:");
      console.log(
        failedXcmPaymentAssets
          .map(({ assetType }) => {
            return `expected: ${assetType} to be present in localAssets `;
          })
          .join(`\n`)
      );

      // Make sure the test fails after we print the errors
      expect(failedXcmPaymentAssets.length, "Failed local assets").to.equal(0);

      // Additional debug logs
      debug(
        `Verified ${foreignXcmAcceptedAssets.length} xcm fee payment assets (at #${atBlockNumber})`
      );
    });
  }
);
