import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";

describeSuite({
  id: "D0802",
  title: "Democracy - genesis",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should have no referendum",
      test: async function () {
        expect((await context.polkadotJs().query.democracy.referendumCount()).toNumber()).to.equal(
          0
        );
      },
    });
    it({
      id: "T02",
      title: "should have no preimages",
      test: async function () {
        expect((await context.polkadotJs().query.preimage.preimageFor.entries()).length).to.equal(
          0
        );
      },
    });
    it({
      id: "T03",
      title: "should have an enactment of 7500",
      test: async function () {
        expect(context.polkadotJs().consts.democracy.enactmentPeriod.toNumber()).to.equal(7500);
      },
    });
    it({
      id: "T04",
      title: "should have a voting period of 36000",
      test: async function () {
        expect(context.polkadotJs().consts.democracy.votingPeriod.toNumber()).to.equal(36000);
      },
    });
    it({
      id: "T05",
      title: "should have a launch period of 7200",
      test: async function () {
        expect(context.polkadotJs().consts.democracy.launchPeriod.toNumber()).to.equal(7200);
      },
    });
    it({
      id: "T06",
      title: "should have a locking period of 7200",
      test: async function () {
        expect(context.polkadotJs().consts.democracy.voteLockingPeriod.toNumber()).to.equal(7200);
      },
    });
    it({
      id: "T07",
      title: "should have a cooloff period of 50400",
      test: async function () {
        expect(context.polkadotJs().consts.democracy.cooloffPeriod.toNumber()).to.equal(50400);
      },
    });
    it({
      id: "T08",
      title: "should have instant fast track allowed",
      test: async function () {
        expect(context.polkadotJs().consts.democracy.instantAllowed.isTrue).to.be.true;
      },
    });
  },
});
