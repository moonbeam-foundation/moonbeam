import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  BALTATHAR_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  DEFAULT_GENESIS_BALANCE,
  DEFAULT_GENESIS_MAPPING,
  baltathar,
  checkBalance,
  generateKeyringPair,
} from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";

describeSuite({
  id: "D020304",
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
        const proposal = context
          .polkadotJs()
          .tx.balances.forceSetBalance(randomAccount.address, 100);
        const encodedProposal = proposal.method.toHex();
        const encodedHash = blake2AsHex(encodedProposal);

        await context.createBlock(
          context.polkadotJs().tx.preimage.notePreimage(encodedProposal).signAsync(baltathar)
        );

        const balanceBefore = await checkBalance(context, BALTATHAR_ADDRESS);
        const call = context
          .polkadotJs()
          .tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS);
        const fee = (await call.paymentInfo(baltathar)).partialFee.toBigInt();

        await context.createBlock(call.signAsync(baltathar));

        const expectedBalance = balanceBefore + existentialDeposit - fee - DEFAULT_GENESIS_MAPPING;
        expect(await checkBalance(context, BALTATHAR_ADDRESS)).toBe(expectedBalance);
      },
    });
  },
});
