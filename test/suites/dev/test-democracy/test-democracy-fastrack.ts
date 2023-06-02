import "@moonbeam-network/api-augment";
import {
  ExtrinsicCreation,
  beforeEach,
  describeSuite,
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  expect,
  notePreimage,
} from "@moonwall/cli";
import { HOURS, alith, baltathar } from "@moonwall/util";
import { SubmittableExtrinsic } from "@polkadot/api/types";
import { Result } from "@polkadot/types";
import { SpRuntimeDispatchError } from "@polkadot/types/lookup";
import { ISubmittableResult } from "@polkadot/types/types";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0801",
  title: "Democracy - FastTracking",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalHash: string;
    let proposal: SubmittableExtrinsic<"promise", ISubmittableResult>;

    beforeEach(async () => {
      proposal = context
        .polkadotJs()
        .tx.parachainStaking.setParachainBondAccount(
          privateKeyToAccount(generatePrivateKey()).address
        );
      proposalHash = await notePreimage(context, proposal, alith);
      await execCouncilProposal(
        context,
        context.polkadotJs().tx.democracy.externalProposeMajority({
          Lookup: {
            hash: proposalHash,
            len: proposal.method.encodedLength,
          },
        })
      );
    });

    it({
      id: "T01",
      title: "should succeed with less than 1/2 of the council",
      test: async function () {
        const referendumCount = (
          await context.polkadotJs().query.democracy.referendumCount()
        ).toNumber();
        const { events }: ExtrinsicCreation = await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.democracy.fastTrack(proposalHash, 4 * HOURS, 1),
          [alith],
          1
        );

        expect(
          events.findIndex(
            ({ event: { section, method } }) =>
              section == "techCommitteeCollective" && method == "Executed"
          ),
          "Technical Committee Wrong event"
        ).to.equal(2);
        expect((events[2].event.data[1] as Result<any, SpRuntimeDispatchError>).isOk).to.be.true;
        expect(
          (await context.polkadotJs().query.democracy.referendumCount()).toNumber(),
          "Unexpected count, should not be sent to referendum"
        ).toBe(referendumCount + 1);
      },
    });

    it({
      id: "T02",
      title: "should succeed with the full council",
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
      title: "should fail with less than 1/2 of the council",
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
  },
});
