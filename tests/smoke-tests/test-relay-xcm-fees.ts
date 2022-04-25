import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:treasury");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify XCM relay weight fees`, { wssUrl, relayWssUrl }, (context) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let relayAtBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let relayApiAt: ApiDecoration<"promise"> = null;

  before("Setup api", async function () {
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );

    relayAtBlockNumber = (await context.relayApi.rpc.chain.getHeader()).number.toNumber();
    relayApiAt = await context.relayApi.at(
      await context.relayApi.rpc.chain.getBlockHash(relayAtBlockNumber)
    );
  });

  it("should have value matching relay", async function () {
    // Load data
    const transactInfo = await apiAt.query.xcmTransactor.transactInfoWithWeightLimit.entries();
    const relayWeights =
      relayApiAt.consts.system.blockWeights.perClass.normal.baseExtrinsic.toBigInt();
    const expectedFeePerSecond = relayWeights; // TODO: Gorka to provide computation

    expect(transactInfo.length).to.be.equal(1);
    expect(transactInfo[0][1].unwrap().feePerSecond.toBigInt()).to.be.equal(expectedFeePerSecond);

    debug(`Verified transactInfoWithWeightLimit with relay baseExtrinsic weight`);
  });
});
