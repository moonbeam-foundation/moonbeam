import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { MultiLocation } from "@polkadot/types/interfaces";
import { it } from "mocha";
const debug = require("debug")("smoke:treasury");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

describeSmokeSuite(`Verify XCM weight fees for relay`, { wssUrl, relayWssUrl }, (context) => {
  const conditionalIt = process.env.SKIP_RELAY_TESTS ? it.skip : it;
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

  conditionalIt("should have value over relay expected fees", async function () {
    // Load data
    const relayRuntime = context.relayApi.runtimeVersion.specName.toString();
    const relayMultiLocation: MultiLocation = context.polkadotApi.createType(
      "MultiLocation",
      JSON.parse('{ "parents": 1, "interior": "Here" }')
    );

    const units = relayRuntime.startsWith("polkadot")
      ? 10_000_000_000n
      : relayRuntime.startsWith("kusama") ||
        relayRuntime.startsWith("rococo") ||
        relayRuntime.startsWith("westend")
      ? 1_000_000_000_000n
      : 1_000_000_000_000n;

    const seconds = 1_000_000_000_000n;

    const cent =
      relayRuntime.startsWith("polkadot") ||
      relayRuntime.startsWith("rococo") ||
      relayRuntime.startsWith("westend")
        ? units / 100n
        : relayRuntime.startsWith("kusama")
        ? units / 30_000n
        : units / 100n;
    const coef = cent / 10n;

    const relayBaseWeight =
      relayApiAt.consts.system.blockWeights.perClass.normal.baseExtrinsic.toBigInt();

    const expectedFeePerSecond = (coef * seconds) / relayBaseWeight;

    const parachainRuntime = context.polkadotApi.runtimeVersion.specVersion.toNumber();

    let feePerSecondValueForRelay;
    if (parachainRuntime >= 1600) {
      feePerSecondValueForRelay = (
        (await apiAt.query.xcmTransactor.destinationAssetFeePerSecond(relayMultiLocation)) as any
      ).unwrap();
    } else {
      feePerSecondValueForRelay = (
        await apiAt.query.xcmTransactor.transactInfoWithWeightLimit(relayMultiLocation)
      ).unwrap().feePerSecond;
    }
    expect(
      feePerSecondValueForRelay.toBigInt() >= expectedFeePerSecond,
      `failed check: feePerSecond: ${feePerSecondValueForRelay} > expected ${expectedFeePerSecond}`
    ).to.be.true;
    expect(
      // Conservative approach to allow up to 2 time the fees
      feePerSecondValueForRelay.toBigInt() < expectedFeePerSecond * 2n,
      `failed check: feePerSecond: ${feePerSecondValueForRelay} < expected ${
        expectedFeePerSecond * 2n
      }`
    ).to.be.true;

    debug(
      `Verified feePerSecond for ${relayMultiLocation} transactInfos ` +
        `within relay base weight range`
    );
  });
});
