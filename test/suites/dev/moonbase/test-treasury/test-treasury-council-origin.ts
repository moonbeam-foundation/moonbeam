import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, baltathar, ethan } from "@moonwall/util";
import type { FrameSupportPalletId } from "@polkadot/types/lookup";

describeSuite({
  id: "D013802",
  title: "Treasury pallet spend_local call (Council Origin)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let treasuryPalletId: FrameSupportPalletId;
    let treasuryAddress: string;
    let api: ApiPromise;
    let assetKind;

    beforeAll(async function () {
      api = context.polkadotJs();
      treasuryPalletId = api.consts.treasury.palletId;
      treasuryAddress = `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`;
    
      assetKind = api.createType("FrameSupportTokensFungibleUnionOfNativeOrWithId", "Native");
      const createRate = api.tx.assetRate.create(
        assetKind,
        api.createType("u128", 1n)
      );
      const sudoCall = api.tx.sudo.sudo(createRate);
      await context.createBlock(sudoCall, { allowFailures: false });
    });

    it({
      id: "T03",
      title: "Members of the treasury council can spend treasury funds",
      test: async function () {
        // Set Alith as member of the treasury council
        const setMembersTX = api.tx.treasuryCouncilCollective.setMembers(
          [alith.address, baltathar.address],
          alith.address,
          3
        );
        await context.createBlock(await api.tx.sudo.sudo(setMembersTX).signAsync(alith), {
          allowFailures: false,
        });

        // Fund treasury
        const treasuryPot = 2_000_000_000_000_000n;
        await context.createBlock(
          await api.tx.balances.transferAllowDeath(treasuryAddress, treasuryPot),
          { allowFailures: false }
        );
        await context.createBlock();
        expect((await api.query.treasury.deactivated()).toBigInt()).toBeGreaterThan(treasuryPot);

        // Pre-checks
        expect((await api.query.treasury.proposalCount()).toNumber()).to.equal(0);
        expect((await api.query.treasury.approvals()).length).to.equal(0);

        // Approve treasury spend to Ethan
        const proposal_value = 1_000_000_000_000_000n;
        const tx = api.tx.treasury.spend(assetKind, proposal_value, ethan.address, null);
        const signedTx = api.tx.treasuryCouncilCollective.propose(2, tx, 1_000).signAsync(alith);
        const blockResult = await context.createBlock(signedTx, {
          allowFailures: false,
          expectEvents: [api.events.treasuryCouncilCollective.Proposed],
        });

        const councilProposalHash = blockResult
          .result!.events.find(({ event: { method } }) => method.toString() === "Proposed")!
          .event.data[2].toHex();

        await context.createBlock(
          api.tx.treasuryCouncilCollective.vote(councilProposalHash, 0, true).signAsync(alith),
          {
            allowFailures: false,
            expectEvents: [api.events.treasuryCouncilCollective.Voted],
          }
        );

        await context.createBlock(
          api.tx.treasuryCouncilCollective.vote(councilProposalHash, 0, true).signAsync(baltathar),
          {
            allowFailures: false,
            expectEvents: [api.events.treasuryCouncilCollective.Voted],
          }
        );

        await context.createBlock(
          api.tx.treasuryCouncilCollective
            .close(
              councilProposalHash,
              0,
              {
                refTime: 50_000_000_000,
                proofSize: 100_000,
              },
              1_000
            )
            .signAsync(alith),
          {
            expectEvents: [
              api.events.treasuryCouncilCollective.Closed,
              api.events.treasury.AssetSpendApproved,
            ],
          }
        );

        // Spending was successfully submitted
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(1);

        await context.createBlock(await api.tx.treasury.payout(0).signAsync(ethan), {
          allowFailures: false,
          expectEvents: [api.events.treasury.Paid],
        });
      },
    });
  },
});
