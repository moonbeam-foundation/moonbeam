import "@moonbeam-network/api-augment";
import { BALTATHAR_SESSION_ADDRESS, generateKeyringPair } from "@moonwall/util";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D010204",
  title: "Author Mapping - Fail without deposit",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      const rando = generateKeyringPair();
      expect((await api.query.system.account(rando.address as string)).data.free.toBigInt()).to.eq(
        0n
      );
      try {
        await api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS).signAndSend(rando);
      } catch (e: any) {
        expect(e.message.toString()).to.eq(
          "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
        );
      }
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should not add the association",
      test: async function () {
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).toBeUndefined();
      },
    });

    // TODO: Fix this test as there is no failed extrinsic in the block
    it({
      id: "T02",
      title: "should check events for failure",
      modifier: "skip",
      test: async function () {
        const signedBlock = await api.rpc.chain.getBlock();
        const allRecords = await api.query.system.events.at(signedBlock.block.header.hash);

        // map between the extrinsics and events
        signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
          // filter the specific events based on the phase and then the
          // index of our extrinsic in the block
          const events = allRecords.filter(
            ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
          );

          switch (index) {
            // First 3 events:
            // timestamp.set:: system.ExtrinsicSuccess
            // parachainUpgrade.setValidationData:: system.ExtrinsicSuccess
            // authorInherent.setAuthor:: system.ExtrinsicSuccess
            case 0:
            case 1:
            case 2:
              expect(events.length === 1 && api.events.system.ExtrinsicSuccess.is(events[0].event))
                .to.be.true;
              break;
            // Fourth extrinsic
            case 3:
              expect(section === "authorMapping" && method === "addAssociation").to.be.true;
              expect(events.length === 6);
              expect(api.events.system.NewAccount.is(events[2].event)).to.be.true;
              expect(api.events.balances.Endowed.is(events[3].event)).to.be.true;
              expect(api.events.system.ExtrinsicFailed.is(events[5].event)).to.be.true;
              break;
            default:
              throw new Error(`Unexpected extrinsic`);
          }
        });
        expect(signedBlock.block.extrinsics).to.be.lengthOf(4);
      },
    });
  },
});
