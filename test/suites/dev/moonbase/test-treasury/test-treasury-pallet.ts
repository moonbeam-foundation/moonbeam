import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, baltathar, ethan } from "@moonwall/util";
import { expectSubstrateEvent } from "../../../../helpers";
import type { FrameSupportPalletId } from "@polkadot/types/lookup";

describeSuite({
  id: "D013801",
  title: "Treasury pallet tests",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let treasuryPalletId: FrameSupportPalletId;
    let treasuryAddress: string;
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      treasuryPalletId = api.consts.treasury.palletId;
      treasuryAddress = `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`;
    });

    it({
      id: "T01",
      title: "Origins that are not Root or members of treasury council cannot spend treasury funds",
      test: async function () {
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);

        // Creates a proposal
        const proposal_value = 1000000000n;
        const tx = api.tx.treasury.spendLocal(proposal_value, ethan.address);
        const signedTx = await tx.signAsync(baltathar);
        await context.createBlock(signedTx, {
          expectEvents: [api.events.system.ExtrinsicFailed],
        });

        expect((await api.query.treasury.proposalCount()).toNumber()).to.equal(0);
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "Root can spend treasury funds",
      test: async function () {
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
        const tx = api.tx.treasury.spendLocal(proposal_value, ethan.address);
        const signedTx = await api.tx.sudo.sudo(tx).signAsync(alith);
        await context.createBlock(signedTx, {
          allowFailures: false,
          expectEvents: [api.events.treasury.SpendApproved],
        });

        // Spending was successfully approved
        expect((await api.query.treasury.proposalCount()).toNumber()).to.equal(1);
        expect((await api.query.treasury.approvals()).length).to.equal(1);
      },
    });

    it({
      id: "T03",
      title: "Members of the treasury council can spend treasury funds",
      test: async function () {
        // Set Alith as member of the treasury council
        const setMembersTX = api.tx.treasuryCouncilCollective.setMembers(
          [alith.address],
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
        const tx = api.tx.treasury.spendLocal(proposal_value, ethan.address);
        const signedTx = api.tx.treasuryCouncilCollective.propose(1, tx, 1_000).signAsync(alith);

        await context.createBlock(signedTx, {
          allowFailures: false,
          expectEvents: [api.events.treasury.SpendApproved],
        });

        // Spending was successfully approved
        expect((await api.query.treasury.proposalCount()).toNumber()).to.equal(1);
        expect((await api.query.treasury.approvals()).length).to.equal(1);
      },
    });
  },
});
