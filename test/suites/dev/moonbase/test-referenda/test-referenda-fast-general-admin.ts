import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, proposeReferendaAndDeposit } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import "@polkadot/api-augment";

describeSuite({
  id: "D013301",
  title: "Referenda - FastGeneralAdmin",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let refIndex: number;
    let proposalHash: string;
    beforeAll(async () => {
      // Just build the arguments. They dont matter that much though, since
      // we will not make sure it executes in the relay
      const transactWeights = context
        .polkadotJs()
        .createType("PalletXcmTransactorTransactWeights", {
          transactRequiredWeightAtMost: { refTime: 10000, proofSize: 10000 },
          overallWeight: { Limited: { refTime: 10000, proofSize: 10000 } },
        });

      const fee = context.polkadotJs().createType("PalletXcmTransactorCurrencyPayment", {
        currency: {
          AsMultiLocation: {
            V3: {
              parents: 1,
              interior: {
                Here: null,
              },
            },
          },
        },
        feeAmount: 10000,
      });

      // The proposal itself
      const proposal = (context.polkadotJs().tx.xcmTransactor as any).hrmpManage(
        { Accept: { para_id: 2000 } },
        fee,
        transactWeights
      );

      // The origin we want to use, post a referenda and deposit.
      [refIndex, proposalHash] = await proposeReferendaAndDeposit(context, alith, proposal, {
        Origins: "FastGeneralAdmin",
      });
    });

    it({
      id: "T01",
      title: "fastGeneralAdmin origin should match to general admin track",
      test: async function () {
        const refInfo = await context.polkadotJs().query.referenda.referendumInfoFor(refIndex);
        const track = refInfo.unwrap().asOngoing.track.toString();
        const tracks = context.polkadotJs().consts.referenda.tracks;
        const trackName = tracks.find(([index, info]) => index.toString() == track)![1].name;

        expect(trackName.toString()).to.be.eq("fast_general_admin");
      },
    });
  },
});
