import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, ethan } from "@moonwall/util";
import type { FrameSupportPalletId } from "@polkadot/types/lookup";

describeSuite({
  id: "D013802",
  title: "Treasury pallet spend_local call (Council Origin)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let treasuryPalletId: FrameSupportPalletId;
    let treasuryAddress: string;
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
      treasuryPalletId = api.consts.treasury.palletId;
      treasuryAddress = `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`;
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
