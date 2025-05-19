import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, baltathar, ethan } from "@moonwall/util";
import type { FrameSupportPalletId } from "@polkadot/types/lookup";

describeSuite({
  id: "D013801",
  title: "Treasury pallet spend call (Root Origin)",
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
      id: "T01",
      title: "Origins that are not Root or members of treasury council cannot spend treasury funds",
      test: async function () {
        const proposal_value = 1000000000n;
        const tx = api.tx.treasury.spend(null, proposal_value, ethan.address, null);
        const signedTx = await tx.signAsync(baltathar);
        await context.createBlock(signedTx, {
          expectEvents: [api.events.system.ExtrinsicFailed],
        });

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
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);

        // Approve treasury spend to Ethan
        const proposal_value = 1_000_000_000_000_000n;
        const tx = api.tx.treasury.spend(null, proposal_value, ethan.address, null);
        const signedTx = await api.tx.sudo.sudo(tx).signAsync(alith);
        await context.createBlock(signedTx, {
          allowFailures: false,
          expectEvents: [api.events.treasury.AssetSpendApproved],
        });

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
