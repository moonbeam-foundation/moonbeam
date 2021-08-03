import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import { ALITH_PRIV_KEY, PROPOSAL_AMOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { execFromTwoThirdsOfCouncil, execFromAllMembersOfTechCommittee } from "../util/governance";

const keyring = new Keyring({ type: "ethereum" });

let alith;

describeDevMoonbeam("Governance", (context) => {
  before("Create accounts", async () => {
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });

  it("should be able to submit a proposal", async function () {
    // Alith submit a proposal
    let proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";
    await context.polkadotApi.tx.democracy
      .propose(proposalHash, PROPOSAL_AMOUNT)
      .signAndSend(alith);
    await context.createBlock();

    // Verify that Alith proposal is registered
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toHuman()).to.equal("1");
  });
  it("should be able to fast track a referundum", async function () {
    // Verify that no referundum is triggered
    expect((await context.polkadotApi.query.democracy.referendumCount()).toHuman()).to.equal("0");

    const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";
    await execFromTwoThirdsOfCouncil(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
    );
    await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 5, 0)
    );

    // Verify that one referundum is triggered
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("1");
  });
});
