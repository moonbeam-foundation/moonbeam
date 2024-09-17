import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { baltathar, ethan, alith } from "@moonwall/util";

describeSuite({
  id: "D013801",
  title: "Treasury pallet tests",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "Non root can not spend from treasury",
      test: async function () {
        const proposal_value = 1000000000n;
        const assetKind = null;
        expect(
          await context
            .createBlock(
              context
                .polkadotJs()
                .tx.treasury.spend(assetKind, proposal_value, baltathar.address, null)
                .signAsync(ethan)
            )
            .catch((e) => e.toString())
        ).to.equal("RpcError: 1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T02",
      title: "Root can spend from treasury",
      test: async function () {
        const api = context.polkadotJs();

        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(0);
        const balanceBefore = (
          await api.query.system.account(baltathar.address)
        ).data.free.toBigInt();

        // Value needs to be higher than the transaction fee paid by dave, but lower than the total treasury pot
        const proposal_value = 1000000000n;
        const assetKind = null;
        const signedTx = api.tx.sudo.sudo(
          api.tx.treasury.spend(assetKind, proposal_value, ethan.address, null)
        );
        const { result } = await context.createBlock([signedTx]);

        console.log(result);
        expect((await api.query.treasury.spendCount()).toNumber()).to.equal(1);

        const tx2 = api.tx.treasury.payout(0);
        const signedTx2 = await tx2.signAsync(ethan);
        await context.createBlock([signedTx2]);

        const balanceAfter = (await api.query.system.account(ethan.address)).data.free.toBigInt();
        expect(balanceAfter).toBeGreaterThan(balanceBefore);
      },
    });
  },
});
