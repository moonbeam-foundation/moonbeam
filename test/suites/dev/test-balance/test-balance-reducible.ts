import "@moonbeam-network/api-augment";
import "@polkadot/api-augment";
import { expect, describeSuite } from "@moonwall/cli";
import {
  alith,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  generateKeyringPair,
} from "@moonwall/util";
import { checkBalance } from "../../../helpers/viem.js";
import { blake2AsHex } from "@polkadot/util-crypto";

describeSuite({
  id: "D030401",
  title: "Reducible Balance",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should show the reducible balanced when some amount is locked",
      test: async function () {
        expect(await checkBalance(context), "Balance should be untouched from genesis amount").toBe(
          ALITH_GENESIS_TRANSFERABLE_BALANCE
        );
        const randomAccount = generateKeyringPair();
        const existentialDeposit = context
          .polkadotJs()
          .consts.balances.existentialDeposit.toBigInt();
        const minDepositAmount = context.polkadotJs().consts.democracy.minimumDeposit.toBigInt();
        const proposal = context
          .polkadotJs()
          .tx.balances.setBalance(randomAccount.address, 100, 100);
        const encodedProposal = proposal.method.toHex();
        const encodedHash = blake2AsHex(encodedProposal);
        await context.createBlock(context.polkadotJs().tx.preimage.notePreimage(encodedProposal));
        const balanceBefore = await checkBalance(context);
        const fee = (
          await context
            .polkadotJs()
            .tx.democracy.propose(
              {
                Lookup: {
                  hash: encodedHash,
                  len: proposal.method.encodedLength,
                },
              },
              minDepositAmount
            )
            .paymentInfo(alith)
        ).partialFee.toBigInt();
        await context.createBlock(
          context.polkadotJs().tx.democracy.propose(
            {
              Lookup: {
                hash: encodedHash,
                len: proposal.method.encodedLength,
              },
            },
            minDepositAmount
          )
        );
        const expectedBalance = balanceBefore - minDepositAmount + existentialDeposit - fee;
        expect(await checkBalance(context)).toBe(expectedBalance);
      },
    });
  },
});
