import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:treasury");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify XCM weight fees for relay`, { wssUrl, relayWssUrl }, (context) => {
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

  it("should have value over relay expected fees", async function () {
    // Load data
    const transactInfos = await apiAt.query.xcmTransactor.transactInfoWithWeightLimit.entries();

    const relayBaseWeight =
      relayApiAt.consts.system.blockWeights.perClass.normal.baseExtrinsic.toBigInt();
    const seconds = 1_000_000_000_000n;
    const cent = seconds / 30_000n;
    const coef = cent / 10n;
    const expectedFeePerSecond = (coef * seconds) / relayBaseWeight;

    expect(transactInfos.length, "Missing transactInfoWithWeightLimit data").to.be.at.least(1);
    for (const transactInfo of transactInfos) {
      const feePerSecond = transactInfo[1].unwrap().feePerSecond.toBigInt();
      expect(
        feePerSecond > expectedFeePerSecond,
        `failed check: feePerSecond: ${feePerSecond} > expected ${expectedFeePerSecond}`
      ).to.be.true;
      expect(
        feePerSecond < (expectedFeePerSecond * 101n) / 100n,
        `failed check: feePerSecond: ${feePerSecond} < expected ${
          (expectedFeePerSecond * 101n) / 100n
        }`
      ).to.be.true;
    }
    debug(
      `Verified feePerSecond for ${transactInfos.length} transactInfos within relay base weight range`
    );
  });
});
