import "@moonbeam-network/api-augment";
import { CHARLETH_ADDRESS, alith, beforeAll, describeSuite, expect } from "moonwall";

import {
  XcmFragment,
  type RawXcmMessage,
  sovereignAccountOfSibling,
  type XcmFragmentConfig,
  injectHrmpMessageAndSeal,
} from "../../../../helpers";
import { parseEther } from "ethers";
import type { ApiPromise } from "@polkadot/api";

// Here we are testing that instructions with finite weights are executed.
// Some instructions (like `UniversalOrigin`) are now intentionally priced with
// `Weight::MAX` in the runtime and are therefore treated as unsupported.
describeSuite({
  id: "D010702",
  title: "XCM - Max Weight Instructions",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let dotAsset: XcmFragmentConfig;
    let amount: bigint;
    const paraId: number = 888;
    let api: ApiPromise;

    beforeAll(async () => {
      api = await context.polkadotJs();

      const paraSovereign = sovereignAccountOfSibling(context, paraId);
      const metadata = await api.rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();

      // Send some native tokens to the sovereign account of paraId (to pay fees)
      await api.tx.balances.transferAllowDeath(paraSovereign, parseEther("1")).signAndSend(alith);
      await context.createBlock();

      amount = 1_000_000_000_000_000n;
      dotAsset = {
        assets: [
          {
            multilocation: {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
            fungible: amount,
          },
        ],
        beneficiary: CHARLETH_ADDRESS,
      };
    });

    it({
      id: "T01",
      title: "UniversalOrigin is treated as unsupported (max weight)",
      test: async function () {
        const xcmMessage = new XcmFragment(dotAsset)
          .withdraw_asset()
          .buy_execution()
          .universal_origin({ GlobalConsensus: "Polkadot" })
          .as_v4();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        const events = (await api.query.system.events())
          .filter(({ event }) => api.events.messageQueue.Processed.is(event))
          .map((e) => e.event.data.toHuman() as { success: boolean });

        // UniversalOrigin is now weighted with Weight::MAX, so the message
        // is rejected during weighing and never reaches the MessageQueue.
        expect(events).to.have.lengthOf(0);
      },
    });
  },
});
