import "@moonbeam-network/api-augment";
import {
  TransactionTypes,
  describeSuite,
  expect,
  extractInfo,
  filterAndApply,
} from "@moonwall/cli";
import { BALTATHAR_ADDRESS, TREASURY_ACCOUNT, createRawTransfer, extractFee } from "@moonwall/util";
import type { u128 } from "@polkadot/types-codec";
import { AccountId20, EventRecord } from "@polkadot/types/interfaces";

describeSuite({
  id: "D1505",
  title: "Fees - Transaction",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "treasury should be empty at genesis",
      test: async () => {
        expect(
          await context.viemClient("public").getBalance({ address: TREASURY_ACCOUNT }),
          "Treasury account should be initially empty"
        ).to.equal(0n);
      },
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 2}`,
        title: `should send 20% of fees to treasury for ${txnType} transfers`,
        test: async () => {
          const balBefore = await context
            .viemClient("public")
            .getBalance({ address: TREASURY_ACCOUNT });
          const issuanceBefore = (
            await context.polkadotJs().query.balances.totalIssuance()
          ).toBigInt();
          const { result } = await context.createBlock(
            await createRawTransfer(context, BALTATHAR_ADDRESS, 128, { type: txnType })
          );

          const balAfter = await context
            .viemClient("public")
            .getBalance({ address: TREASURY_ACCOUNT });
          const issuanceAfter = (
            await context.polkadotJs().query.balances.totalIssuance()
          ).toBigInt();

            

          const treasuryIncrease = balAfter - balBefore;
          const fee = extractFee(result!.events)!.amount.toBigInt();
          expect(fee / treasuryIncrease, "20% of the fees should go to treasury").to.equal(5n);

          const issuanceDecrease = issuanceBefore - issuanceAfter;
          expect((fee * 100n) / issuanceDecrease, "80% of the fees should be burned").to.equal(
            125n
          );
        },
      });
    }
  },
});