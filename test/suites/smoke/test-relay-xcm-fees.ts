import "@moonbeam-network/api-augment";
import type { ApiDecoration } from "@polkadot/api/types";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { extractWeight } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S20",
  title: "Verify XCM weight fees for relay",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber = 0;
    let relayAtBlockNumber = 0;
    let paraApiAt: ApiDecoration<"promise">;
    let relayApiAt: ApiDecoration<"promise">;
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      relayApi = context.polkadotJs("relay");

      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      paraApiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));

      relayAtBlockNumber = (await relayApi.rpc.chain.getHeader()).number.toNumber();
      relayApiAt = await relayApi.at(await relayApi.rpc.chain.getBlockHash(relayAtBlockNumber));
    });

    it({
      id: "C100",
      title: "should have value over relay expected fees",
      test: async function () {
        const relayRuntime = relayApi.runtimeVersion.specName.toString();
        const paraRuntime = paraApi.runtimeVersion.specName.toString();
        const relayVersion = relayApi.runtimeVersion.specVersion.toNumber();

        // skip test if runtime inconsistency. The storage is set for
        // specific runtimes, so does not make sense to compare non-matching runtimes
        const skipTestRuntimeInconsistency = !(
          (relayRuntime.startsWith("polkadot") && paraRuntime.startsWith("moonbeam")) ||
          (relayRuntime.startsWith("kusama") && paraRuntime.startsWith("moonriver")) ||
          (relayRuntime.startsWith("westend") && paraRuntime.startsWith("moonbase"))
        );

        if (skipTestRuntimeInconsistency) {
          log(`Relay and Para runtimes dont match, skipping test`);
          return;
        }

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

        const parachainRuntime = paraApi.runtimeVersion.specVersion.toNumber();

        let feePerSecondValueForRelay;
        if (parachainRuntime >= 1600) {
          feePerSecondValueForRelay = (
            await paraApiAt.query.xcmTransactor.destinationAssetFeePerSecond({
              parents: 1,
              interior: "Here",
            })
          ).unwrap();
        } else {
          feePerSecondValueForRelay = (
            (await paraApiAt.query.xcmTransactor.transactInfoWithWeightLimit({
              parents: 1,
              interior: "Here",
            })) as any
          ).unwrap().feePerSecond;
        }
        expect(
          feePerSecondValueForRelay.toBigInt() >= expectedFeePerSecond,
          "failed check: feePerSecond: " +
            `${feePerSecondValueForRelay} > expected ${expectedFeePerSecond}`
        ).to.be.true;
        expect(
          // Conservative approach to allow up to 2 time the fees
          feePerSecondValueForRelay.toBigInt() < expectedFeePerSecond * 2n,
          `failed check: feePerSecond: ${feePerSecondValueForRelay} < expected ${
            expectedFeePerSecond * 2n
          }`
        ).to.be.true;

        log(
          `Verified feePerSecond for relayMultiLocation transactInfos ` +
            `within relay base weight range`
        );
      },
    });
  },
});
