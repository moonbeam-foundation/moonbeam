import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:author");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify author filter consistency`, { wssUrl, relayWssUrl }, (context) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let specVersion: number = 0;

  before("Setup api", async function () {
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    specVersion = (await apiAt.query.system.lastRuntimeUpgrade()).unwrap().specVersion.toNumber();
  });

  it("should have eligibility > 0", async function () {
    if (specVersion < 1500) {
      const eligibilityRatio = await apiAt.query.authorFilter.eligibleRatio();
      expect(eligibilityRatio.toBigInt() > 0n).to.be.true;
    }
    if (specVersion >= 1500) {
      // TODO remove `as any` once api-augment is updated
      const eligibilityCount = (await apiAt.query.authorFilter.eligibleCount()) as any;
      expect(eligibilityCount.toNumber() > 0).to.be.true;
    }
    debug(`Verified eligibility`);
  });
});
