import { describeSuite, expect, TransactionTypes } from "@moonwall/cli";
import {
  alith,
  baltathar,
  BALTATHAR_ADDRESS,
  createRawTransfer,
  extractFee,
  Perbill,
  TREASURY_ACCOUNT,
} from "@moonwall/util";
import { parameterType, UNIT } from "./test-parameters";
import { BN } from "@polkadot/util";

interface TestCase {
  proportion: Perbill;

  transfer_amount: bigint;
  tipAmount: bigint;
}

describeSuite({
  id: "DTemp03",
  title: "Parameters - RuntimeConfig",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let testCounter = 0;

    let testCases: TestCase[] = [
      {
        proportion: new Perbill(0),
        transfer_amount: 10n * UNIT,
        tipAmount: 1n * UNIT,
      },
      {
        proportion: new Perbill(1, 100),
        transfer_amount: 1000n,
        tipAmount: 100n,
      },
      {
        proportion: new Perbill(355, 1000),
        transfer_amount: 5n * UNIT,
        tipAmount: 111112n,
      },
      {
        proportion: new Perbill(400, 1000),
        transfer_amount: 10n * UNIT,
        tipAmount: 1n * UNIT,
      },
      {
        proportion: new Perbill(500, 1000),
        transfer_amount: 10n * UNIT,
        tipAmount: 1n * UNIT,
      },
      {
        proportion: new Perbill(963, 1000),
        transfer_amount: 10n * UNIT,
        tipAmount: 128n,
      },
      {
        proportion: new Perbill(99, 100),
        transfer_amount: 10n * UNIT,
        tipAmount: 3n,
      },
      {
        proportion: new Perbill(1, 1),
        transfer_amount: 10n * UNIT,
        tipAmount: 32n,
      },
    ];

    for (const t of testCases) {
      const burnProportion = new Perbill(new BN(1e9).sub(t.proportion.value()));

      const treasuryPercentage = t.proportion.value().toNumber() / 1e7;
      const burnPercentage = burnProportion.value().toNumber() / 1e7;

      const calcTreasuryIncrease = (feeWithTip: bigint): bigint => {
        let val = t.proportion.of(new BN(feeWithTip));
        return BigInt(val.toString());
      };
      const calcIssuanceDecrease = (feeWithTip: bigint): bigint => {
        let val = burnProportion.of(new BN(feeWithTip));
        return BigInt(val.toString());
      };

      for (const txnType of TransactionTypes) {
        it({
          id: `T${++testCounter}`,
          title: `Changing FeesTreasuryProportion to ${treasuryPercentage}% for Ethereum ${txnType} transfers`,
          test: async () => {
            const param = parameterType(
              context,
              "RuntimeConfig",
              "FeesTreasuryProportion",
              t.proportion.value()
            );
            await context.createBlock(
              context
                .polkadotJs()
                .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param.toU8a()))
                .signAsync(alith),
              { allowFailures: false }
            );

            const balBefore = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
            const issuanceBefore = (
              await context.polkadotJs().query.balances.totalIssuance()
            ).toBigInt();
            const { result } = await context.createBlock(
              await createRawTransfer(context, BALTATHAR_ADDRESS, t.transfer_amount, {
                type: txnType,
              })
            );

            const balAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
            const issuanceAfter = (
              await context.polkadotJs().query.balances.totalIssuance()
            ).toBigInt();

            const treasuryIncrease = balAfter - balBefore;
            const issuanceDecrease = issuanceBefore - issuanceAfter;
            const fee = extractFee(result?.events)!.amount.toBigInt();

            expect(
              treasuryIncrease,
              `${treasuryPercentage}% of the fees should go to treasury`
            ).to.equal(calcTreasuryIncrease(fee));

            expect(issuanceDecrease, `${burnPercentage}% of the fees should be burned`).to.equal(
              calcIssuanceDecrease(fee)
            );
          },
        });
      }

      for (const withTip of [false]) {
        it({
          id: `T${++testCounter}`,
          title: `Changing FeesTreasuryProportion to ${treasuryPercentage}% for Substrate based transactions with ${
            withTip ? "" : "no "
          }tip`,
          test: async () => {
            const param = parameterType(
              context,
              "RuntimeConfig",
              "FeesTreasuryProportion",
              t.proportion.value()
            );
            await context.createBlock(
              context
                .polkadotJs()
                .tx.sudo.sudo(context.polkadotJs().tx.parameters.setParameter(param.toU8a()))
                .signAsync(alith),
              { allowFailures: false }
            );

            const balanceBefore = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
            const issuanceBefore = (
              await context.polkadotJs().query.balances.totalIssuance()
            ).toBigInt();

            const { result } = await context.createBlock(
              context
                .polkadotJs()
                .tx.balances.transferKeepAlive(alith.address, t.transfer_amount)
                .signAsync(baltathar, { tip: withTip ? t.tipAmount : 0n }),
              { allowFailures: false }
            );

            const balanceAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
            const issuanceAfter = (
              await context.polkadotJs().query.balances.totalIssuance()
            ).toBigInt();

            const treasuryIncrease = balanceAfter - balanceBefore;
            const issuanceDecrease = issuanceBefore - issuanceAfter;
            const fee = extractFee(result?.events)!.amount.toBigInt();

            expect(
              treasuryIncrease,
              `${treasuryPercentage}% of the fees should go to treasury`
            ).to.equal(calcTreasuryIncrease(fee));

            expect(issuanceDecrease, `${burnPercentage}% of the fees should be burned`).to.equal(
              calcIssuanceDecrease(fee)
            );
          },
        });
      }
    }
  },
});
