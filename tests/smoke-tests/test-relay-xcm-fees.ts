import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { MultiLocation } from "@polkadot/types/interfaces";
import { it } from "mocha";
import { extractWeight } from "../util/block";
const debug = require("debug")("smoke:treasury");

describeSmokeSuite("S1800", `Verify XCM weight fees for relay`, (context, testIt) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let relayAtBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;
  let relayApiAt: ApiDecoration<"promise"> = null;

  before("Setup api", async function () {
    if (process.env.SKIP_RELAY_TESTS) {
      this.skip();
    }
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );

    relayAtBlockNumber = (await context.relayApi.rpc.chain.getHeader()).number.toNumber();
    relayApiAt = await context.relayApi.at(
      await context.relayApi.rpc.chain.getBlockHash(relayAtBlockNumber)
    );
  });

  testIt("C100", `should have value over relay expected fees`, async function () {
    // Load data
    const relayRuntime = context.relayApi.runtimeVersion.specName.toString();
    const paraRuntime = context.polkadotApi.runtimeVersion.specName.toString();
    const relayVersion = context.relayApi.runtimeVersion.specVersion.toNumber();

    // skip test if runtime inconsistency. The storage is set for
    // specific runtimes, so does not make sense to compare non-matching runtimes
    let skipTestRuntimeInconsistency =
      (relayRuntime.startsWith("polkadot") && paraRuntime.startsWith("moonbeam")) ||
      (relayRuntime.startsWith("kusama") && paraRuntime.startsWith("moonriver")) ||
      (relayRuntime.startsWith("westend") && paraRuntime.startsWith("moonbase"))
        ? false
        : true;

    if (skipTestRuntimeInconsistency) {
      debug(`Relay and Para runtimes dont match, skipping test`);
      return;
    }
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
        ? units / 3_000n
        : units / 100n;
    const coef = cent / 10n;

    const relayBaseWeight = extractWeight(
      relayApiAt.consts.system.blockWeights.perClass.normal.baseExtrinsic
    ).toBigInt();

    const expectedFeePerSecond = (coef * seconds) / relayBaseWeight;

    const parachainRuntime = context.polkadotApi.runtimeVersion.specVersion.toNumber();

    let feePerSecondValueForRelay;
    if (parachainRuntime >= 1600) {
      feePerSecondValueForRelay = (
        (await apiAt.query.xcmTransactor.destinationAssetFeePerSecond(relayMultiLocation)) as any
      ).unwrap();
    } else {
      feePerSecondValueForRelay = (
        (await apiAt.query.xcmTransactor.transactInfoWithWeightLimit(relayMultiLocation)) as any
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
