import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import { Event } from "@polkadot/types/interfaces";
import {
  ALITH_PRIV_KEY,
  DOROTHY,
  DOROTHY_PRIV_KEY,
  ETHAN,
  ETHAN_PRIVKEY,
  VOTE_AMOUNT,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { execFromTwoThirdsOfCouncil, execFromAllMembersOfTechCommittee } from "../util/governance";

const keyring = new Keyring({ type: "ethereum" });
const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";

let alith;
let dorothy;
let ethan;

describeDevMoonbeam("Proxing governance", (context) => {
  before("Create accounts and fast-tracking referundum", async () => {
    alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    dorothy = await keyring.addFromUri(DOROTHY_PRIV_KEY, null, "ethereum");
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");

    await execFromTwoThirdsOfCouncil(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
    );
    let { events } = await execFromAllMembersOfTechCommittee(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 5, 0)
    );
  });

  it("should be able to vote on behalf of the delegate account", async function () {
    // Verify that one referundum is triggered
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toHuman()).to.equal("1");

    // Dorothy add proxy rigth to ethan for governance only
    await context.polkadotApi.tx.proxy.addProxy(ETHAN, "Governance", 0).signAndSend(dorothy);
    await context.createBlock();

    // Ethan vote as Dorothy
    let voteCall = context.polkadotApi.tx.democracy.vote(0, {
      Standard: { balance: VOTE_AMOUNT, vote: { aye: true, conviction: 1 } },
    });
    await context.polkadotApi.tx.proxy.proxy(DOROTHY, "Governance", voteCall).signAndSend(ethan);
    await context.createBlock();

    // verify events
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      console.log(method, section);
      const events: Event[] = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => {
          console.log(event.toHuman());
          return event;
        });

      switch (index) {
        case 0:
        case 1:
        case 2:
          expect(
            events.length === 1 && context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0])
          ).to.be.true;
          break;
        // Fourth extrinsic: proxy.proxy
        case 3:
          expect(section === "proxy" && method === "proxy").to.be.true;
          expect(events.length === 3);
          expect(context.polkadotApi.events.proxy.ProxyExecuted.is(events[0])).to.be.true;
          expect(Object.keys(events[0].toHuman().data[0])[0] === "Ok").to.be.true;
          expect(context.polkadotApi.events.treasury.Deposit.is(events[1])).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[2])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });

    // Verify that dorothy tokens are used
    let dorothyAccountData = await context.polkadotApi.query.system.account(DOROTHY);
    expect((dorothyAccountData.toHuman() as any).data.free).to.equal("1.2089 MUNIT");

    // Verify that vote is registered
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect((referendumInfoOf.toHuman() as any).Ongoing.proposalHash).to.equal(proposalHash);
    expect((referendumInfoOf.toHuman() as any).Ongoing.tally.ayes).to.equal("10.0000 UNIT");
    expect((referendumInfoOf.toHuman() as any).Ongoing.tally.turnout).to.equal("10.0000 UNIT");
  });
});
