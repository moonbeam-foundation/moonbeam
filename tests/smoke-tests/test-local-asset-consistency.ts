import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { Option, u128 } from "@polkadot/types-codec";
import type { PalletAssetManagerAssetInfo, PalletAssetsAssetDetails } from "@polkadot/types/lookup";

import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { StorageKey } from "@polkadot/types";
const debug = require("debug")("smoke:localAssets");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(
  `Verify local asset count, assetIds and deposits`,
  { wssUrl, relayWssUrl },
  (context) => {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;
    let localAssetDeposits: StorageKey<[u128]>[] = null;
    let localAssetInfo: StorageKey<[u128]>[] = null;
    let localAssetCounter: number = 0;

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
      localAssetDeposits = await apiAt.query.assetManager.localAssetDeposit.keys();
      localAssetCounter = await (await apiAt.query.assetManager.localAssetCounter()).toNumber();
      localAssetInfo = await apiAt.query.assetManager.localAssetDeposit.keys();
    });

    it("should match asset deposit entries with number of assets", async function () {
      expect(
        localAssetDeposits.length,
        `Number of local asset deposits does not much number of local assets`
      ).to.be.eq(localAssetInfo.length);

      debug(
        `Verified number of deposits and local asset number matches: ${localAssetDeposits.length}`
      );
    });

    it("should ensure localAssetCounter is higher than number of local assets", async function () {
      expect(
        localAssetCounter,
        `Local asset counter lower than total local assets`
      ).to.be.greaterThanOrEqual(localAssetInfo.length);

      debug(
        `Verified local asset counter (${localAssetCounter}) 
        >= total local assets: (${localAssetInfo.length})`
      );
    });

    it("assetIds stored by assetManager are also created in LocalAssets", async function () {
      // Instead of putting an expect in the loop. We track all failed entries instead
      const failedLocalAssets: { assetId: string }[] = [];

      const registeredLocalAssetDeposits = localAssetDeposits.map((set) => set.toHex().slice(-32));

      const registeredLocalAssetInfos = localAssetInfo.map((set) => set.toHex().slice(-32));

      for (const assetId of registeredLocalAssetDeposits) {
        if (!registeredLocalAssetInfos.includes(assetId)) {
          failedLocalAssets.push({ assetId: assetId });
        }
      }

      console.log("Failed local asset deposits with non-existent local assets:");
      console.log(
        failedLocalAssets
          .map(({ assetId }) => {
            return `expected: ${assetId} to be present in localAssets `;
          })
          .join(`\n`)
      );

      // Make sure the test fails after we print the errors
      expect(failedLocalAssets.length, "Failed local assets").to.equal(0);

      // Additional debug logs
      debug(
        `Verified ${Object.keys(localAssetInfo).length} total loacl assetIds (at #${atBlockNumber})`
      );
    });
  }
);
