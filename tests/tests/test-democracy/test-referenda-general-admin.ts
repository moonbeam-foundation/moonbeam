import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { alith } from "../../util/accounts";

import { proposeReferendaAndDeposit } from "../../util/governance";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Referenda - GeneralAdmin", (context) => {
  let refIndex: Number;
  let proposalHash: String;
  before("Prepare pre-image and proposal and 3 members TC", async () => {
    // Just build the arguments. They dont matter that much though, since
    // we will not make sure it executes in the relay
    const transactWeights = context.polkadotApi.createType("PalletXcmTransactorTransactWeights", {
      transactRequiredWeightAtMost: { refTime: 10000, proofSize: 10000 },
      overallWeight: { refTime: 10000, proofSize: 10000 },
    });

    let fee = context.polkadotApi.createType("PalletXcmTransactorCurrencyPayment", {
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
    }) as any;

    // The proposal itself
    const proposal = (context.polkadotApi.tx.xcmTransactor as any).hrmpManage(
      { Accept: { para_id: 2000 } },
      fee,
      transactWeights
    ) as any;

    // The origin we want to use, post a referenda and deposit.
    [refIndex, proposalHash] = await proposeReferendaAndDeposit(context, alith, proposal, {
      Origins: "GeneralAdmin",
    });
  });

  it("generalAdmin origin should match to general admin track", async function () {
    let refInfo = (await context.polkadotApi.query.referenda.referendumInfoFor(refIndex)) as any;
    const track = refInfo.unwrap().asOngoing.track;
    const tracks = await context.polkadotApi.consts.referenda.tracks;
    const trackName = tracks.find(([index, info]) => index.toString() == track)[1].name;

    expect(trackName.toString()).to.be.eq("general_admin");
  });
});
