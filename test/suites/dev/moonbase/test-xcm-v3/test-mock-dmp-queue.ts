import "@moonbeam-network/api-augment";
import { customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { GLMR } from "@moonwall/util";
import type { XcmVersionedXcm } from "@polkadot/types/lookup";
import { u8aToHex } from "@polkadot/util";
import { XcmFragment, weightMessage } from "../../../../helpers";

describeSuite({
  id: "D014008",
  title: "Test DMP migration (This test should be removed in RT3000",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "Should test migration to Message Queue",
      test: async function () {
        await context.createBlock();

        let events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.dmpQueue.StartedExport.is(event)
        );
        expect(events).to.have.lengthOf(1);
        
        // Create new block
        await context.createBlock();

        events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.dmpQueue.CompletedExport.is(event)
        );
        expect(events).to.have.lengthOf(1);

        // Create new block
        await context.createBlock();

        events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.dmpQueue.StartedOverweightExport.is(event)
        );
        expect(events).to.have.lengthOf(1);

        // Create new block
        await context.createBlock();

        events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.dmpQueue.CompletedOverweightExport.is(event)
        );
        expect(events).to.have.lengthOf(1);

        // Create new block
        await context.createBlock();

        events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.dmpQueue.StartedCleanup.is(event)
        );
        expect(events).to.have.lengthOf(1);

        // Create new block
        await context.createBlock();

        events = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.dmpQueue.Completed.is(event)
        );
        expect(events).to.have.lengthOf(1);
      },
    });
  },
});
