import {describeSuite, expect, TransactionTypes} from "@moonwall/cli";
import {
  alith,
  BALTATHAR_ADDRESS,
  createRawTransfer,
  extractFee,
  TREASURY_ACCOUNT
} from "@moonwall/util";
import {fail} from "assert";
import {parameterType, UNIT} from "./test-parameters";

describeSuite({
  id: "DTemp02",
  title: "Parameters - RuntimeConfig",
  foundationMethods: "dev",
  testCases: ({it, context, log,}) => {
    let testCounter = 0;


    it({
      id: `T${++testCounter}`,
      title: "treasury should be empty at genesis",
      test: async () => {
        expect(
          await context.viem().getBalance({address: TREASURY_ACCOUNT}),
          "Treasury account should be initially empty"
        ).to.equal(0n);
      },
    });


    for (const txnType of TransactionTypes) {
      it({
        id: `T${++testCounter}`,
        title: `should send 0% of fees to treasury for ${txnType} transfers`,
        test: async () => {
          const param = parameterType(context, "RuntimeConfig", "FeesTreasuryProportion", 0);
          await context.createBlock(
            context
              .polkadotJs()
              .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param.toU8a()))
              .signAsync(alith),
            {allowFailures: false}
          );

          const balBefore = await context.viem().getBalance({address: TREASURY_ACCOUNT});
          const issuanceBefore = (
            await context.polkadotJs().query.balances.totalIssuance()
          ).toBigInt();
          const {result} = await context.createBlock(
            await createRawTransfer(context, BALTATHAR_ADDRESS, 128, {type: txnType})
          );

          const balAfter = await context.viem().getBalance({address: TREASURY_ACCOUNT});
          const issuanceAfter = (
            await context.polkadotJs().query.balances.totalIssuance()
          ).toBigInt();

          const treasuryIncrease = balAfter - balBefore;
          const fee = extractFee(result?.events)!.amount.toBigInt();
          expect(fee / treasuryIncrease, "0% of the fees should go to treasury").to.equal(0n);

          const issuanceDecrease = issuanceBefore - issuanceAfter;
          expect((fee * 100n) / issuanceDecrease, "100% of the fees should be burned").to.equal(
            100n
          );
        },
      });
    }
  },
});