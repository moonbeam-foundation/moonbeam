import "@moonbeam-network/api-augment";
import type { ApiDecoration } from "@polkadot/api/types";
import { beforeAll, describeSuite, expect } from "moonwall";
import type { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "S21",
  title: "Verify XCM weight fees for relay",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let atBlockNumber = 0;
    let paraApiAt: ApiDecoration<"promise">;
    let paraApi: ApiPromise;
    let relayApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      relayApi = context.polkadotJs("relay");

      atBlockNumber = (await paraApi.rpc.chain.getHeader()).number.toNumber();
      paraApiAt = await paraApi.at(await paraApi.rpc.chain.getBlockHash(atBlockNumber));
    });

    it({
      id: "C100",
      title: "should register relay asset as active with positive relativePrice in xcmWeightTrader.supportedAssets",
      test: async function () {
        const relayRuntime = relayApi.runtimeVersion.specName.toString();
        const paraRuntime = paraApi.runtimeVersion.specName.toString();

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

        // The old xcmTransactor.destinationAssetFeePerSecond storage has been removed
        // and replaced by xcmWeightTrader.supportedAssets, which stores (isActive, relativePrice).
        // The relativePrice represents the foreign asset price relative to the native asset,
        // scaled by 10^18 (RELATIVE_PRICE_DECIMALS).
        const relayLocation = {
          parents: 1,
          interior: "Here",
        };

        const supportedAsset = (await paraApiAt.query.xcmWeightTrader.supportedAssets(
          relayLocation
        )) as any;

        expect(
          supportedAsset.isSome,
          "Relay asset location {parents:1, interior:Here} should be registered" +
            " in xcmWeightTrader.supportedAssets"
        ).to.be.true;

        const [isActive, relativePrice] = supportedAsset.unwrap();

        expect(isActive.isTrue, "Relay asset should be active (enabled) in xcmWeightTrader").to.be
          .true;

        expect(
          relativePrice.toBigInt() > 0n,
          "Relay asset relative price should be greater than zero"
        ).to.be.true;

        log(
          `Verified relay asset in xcmWeightTrader.supportedAssets: ` +
            `active=${isActive}, relativePrice=${relativePrice}`
        );
      },
    });
  },
});
