import "@moonbeam-network/api-augment";
import {
  beforeAll,
  describeSuite,
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  expect,
  notePreimage,
} from "@moonwall/cli";
import { GLMR, MIN_GLMR_DELEGATOR, alith, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "D2979",
  title: "Staking - Locks - democracy vote",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const randomAccount = generateKeyringPair();

    beforeAll(async function () {
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(randomAccount.address, MIN_GLMR_DELEGATOR + GLMR),
        { allowFailures: false }
      );

      await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegate(alith.address, MIN_GLMR_DELEGATOR, 0, 0)
          .signAsync(randomAccount),
        { allowFailures: false }
      );

      const proposal = context
        .polkadotJs()
        .tx.parachainStaking.setParachainBondAccount(alith.address);
      const proposalHash = await notePreimage(context, proposal, alith);
      await execCouncilProposal(
        context,
        context.polkadotJs().tx.democracy.externalProposeMajority({
          LookUp: {
            hash: proposalHash,
            len: proposal.encodedLength,
          },
        } as any)
      );
      await execTechnicalCommitteeProposal(
        context,
        context.polkadotJs().tx.democracy.fastTrack(proposalHash, 100, 1)
      );
    });

    it({
      id: "T01",
      title: "should be usable for democracy vote",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.democracy.vote(0, {
              Standard: { balance: MIN_GLMR_DELEGATOR, vote: { aye: true, conviction: 1 } },
            })
            .signAsync(randomAccount)
        );
        expect(result!.successful).to.be.true;
        expect(result!.events.find(({ event: { method } }) => method === "Voted")).to.not.be
          .undefined;
      },
    });
  },
});
