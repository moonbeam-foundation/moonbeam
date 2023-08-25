import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  DEFAULT_GENESIS_BALANCE,
  baltathar,
  checkBalance,
  generateKeyringPair,
} from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";

describeSuite({
  id: "D0304",
  title: "Reducible Balance",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should show the reducible balanced when some amount is locked",
      test: async function () {
        expect(
          await checkBalance(context, BALTATHAR_ADDRESS),
          "Balance should be untouched from genesis amount"
        ).toBe(DEFAULT_GENESIS_BALANCE);
        const randomAccount = generateKeyringPair();
        const existentialDeposit = context
          .polkadotJs()
          .consts.balances.existentialDeposit.toBigInt();
        const minDepositAmount = context.polkadotJs().consts.democracy.minimumDeposit.toBigInt();
        const proposal = context
          .polkadotJs()
          .tx.balances.forceSetBalance(randomAccount.address, 100);
        const encodedProposal = proposal.method.toHex();
        const encodedHash = blake2AsHex(encodedProposal);

        await context.createBlock(
          context.polkadotJs().tx.preimage.notePreimage(encodedProposal).signAsync(baltathar)
        );

        const balanceBefore = await checkBalance(context, BALTATHAR_ADDRESS);
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
            .paymentInfo(baltathar)
        ).partialFee.toBigInt();

        await context.createBlock(
          context
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
            .signAsync(baltathar)
        );

        const expectedBalance = balanceBefore - minDepositAmount + existentialDeposit - fee;
        expect(await checkBalance(context, BALTATHAR_ADDRESS)).toBe(expectedBalance);
      },
    });
  },
});
