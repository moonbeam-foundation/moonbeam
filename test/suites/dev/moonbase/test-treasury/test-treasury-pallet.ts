import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { alith, baltathar, ethan } from "@moonwall/util";
import { expectSubstrateEvent } from "../../../../helpers";

describeSuite({
  id: "D013801",
  title: "Treasury pallet tests",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let api: ApiPromise;

    beforeAll(async function () {
      api = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "Non root cannot spend (local)",
      test: async function () {
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);

        // Creates a proposal
        const proposal_value = 1000000000n;
        const tx = api.tx.treasury.spendLocal(proposal_value, ethan.address);
        const signedTx = await tx.signAsync(baltathar);
        const blockResult = await context.createBlock(signedTx);

        expectSubstrateEvent(blockResult, "system", "ExtrinsicFailed");

        expect((await api.query.treasury.proposalCount()).toNumber()).to.equal(0);
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "Root should be able to spend (local) and approve a proposal",
      timeout: -1,
      test: async function () {
        const spendPeriod = api.consts.treasury.spendPeriod.toNumber();

        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);
        // Creates a proposal
        // Value needs to be higher than the transaction fee paid by ethan,
        // but lower than the total treasury pot
        const proposal_value = 1000000000n;
        const tx = api.tx.treasury.spendLocal(proposal_value, ethan.address);
        const signedTx = await api.tx.sudo.sudo(tx).signAsync(alith);
        const blockResult = await context.createBlock(signedTx, { allowFailures: false });

        expectSubstrateEvent(blockResult, "treasury", "SpendApproved");
        expect((await api.query.treasury.proposalCount()).toNumber()).to.equal(1);
      },
    });
  },
});
