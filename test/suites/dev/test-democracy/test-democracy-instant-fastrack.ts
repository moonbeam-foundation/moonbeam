import "@moonbeam-network/api-augment";
import {
  DevModeContext,
  ExtrinsicCreation,
  beforeEach,
  describeSuite,
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  expect,
  notePreimage,
} from "@moonwall/cli";
import { ALITH_ADDRESS, alith, baltathar, charleth } from "@moonwall/util";
import { Result } from "@polkadot/types";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0803",
  title: "Democracy - Instant FastTracking",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let proposalHash: string;

    beforeEach(async () => {
      proposalHash = await setupProposalAnd3TechnicalCommittee(context);
    });

    it({
      id: "T01",
      title: "should fail with less than 2/3rd of the council",
      test: async function () {
        const referendumCount = (
          await context.polkadotJs().query.democracy.referendumCount()
        ).toNumber();

        const { events }: ExtrinsicCreation = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.democracy.fastTrack(proposalHash, 1, 1),
          [alith],
          1
        );

        expect(
          events.findIndex(
            ({ event: { section, method } }) =>
              section == "techCommitteeCollective" && method == "Executed"
          ),
          "Technical Committee Wrong event"
        ).to.equal(1);
        expect((events[1].event.data[1] as Result<any, SpRuntimeDispatchError>).asErr.isBadOrigin)
          .to.be.true;

        expect(
          (await context.polkadotJs().query.democracy.referendumCount()).toNumber(),
          "Unexpected count, should not be sent to referendum"
        ).toBe(referendumCount);
      },
    });

    it({
      id: "T02",
      title: "should succeed with than 2/3rd of the council",
      test: async function () {
        const referendumCount = (
          await context.polkadotJs().query.democracy.referendumCount()
        ).toNumber();

        const { events }: ExtrinsicCreation = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.democracy.fastTrack(proposalHash, 1, 1),
          [alith, baltathar],
          2
        );

        expect(
          events.findIndex(
            ({ event: { section, method } }) =>
              section == "techCommitteeCollective" && method == "Executed"
          ),
          "Technical Committee Wrong event"
        ).to.equal(4);
        expect((events[4].event.data[1] as Result<any, SpRuntimeDispatchError>).isOk).to.be.true;

        expect(
          (await context.polkadotJs().query.democracy.referendumCount()).toNumber(),
          "Unexpected count, should not be sent to referendum"
        ).toBe(referendumCount + 1);
      },
    });

    it({
      id: "T03",
      title: "should succeed with the full council",
      test: async function () {
        const referendumCount = (
          await context.polkadotJs().query.democracy.referendumCount()
        ).toNumber();

        const { events }: ExtrinsicCreation = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.democracy.fastTrack(proposalHash, 1, 1),
          [alith, baltathar, charleth],
          3
        );

        expect(
          events.findIndex(
            ({ event: { section, method } }) =>
              section == "techCommitteeCollective" && method == "Executed"
          ),
          "Technical Committee Wrong event"
        ).to.equal(4);
        expect((events[4].event.data[1] as Result<any, SpRuntimeDispatchError>).isOk).to.be.true;

        expect(
          (await context.polkadotJs().query.democracy.referendumCount()).toNumber(),
          "Unexpected count, should not be sent to referendum"
        ).to.be.equal(referendumCount + 1);
      },
    });
  },
});

const setupProposalAnd3TechnicalCommittee = async (context: DevModeContext) => {
  const proposal = context
    .polkadotJs()
    .tx.parachainStaking.setParachainBondAccount(privateKeyToAccount(generatePrivateKey()).address);
  let proposalHash = await notePreimage(context, proposal, alith);
  await execCouncilProposal(
    context,
    context.polkadotJs().tx.democracy.externalProposeMajority({
      Lookup: {
        hash: proposalHash,
        len: proposal.method.encodedLength,
      },
    })
  );

  await context.createBlock(
    context
      .polkadotJs()
      .tx.sudo.sudo(
        context
          .polkadotJs()
          .tx.techCommitteeCollective.setMembers(
            [ALITH_ADDRESS, baltathar.address, charleth.address],
            ALITH_ADDRESS,
            (
              await context.polkadotJs().query.techCommitteeCollective.members()
            ).length
          )
      )
  );
  return proposalHash;
};
