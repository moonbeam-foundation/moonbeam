import { describeSuite, expect, TransactionTypes } from "@moonwall/cli";
import {
  alith,
  baltathar,
  BALTATHAR_ADDRESS,
  createRawTransfer,
  extractFee,
  TREASURY_ACCOUNT,
} from "@moonwall/util";
import { parameterType } from "./test-parameters";

describeSuite({
  id: "DTemp02",
  title: "Parameters - RuntimeConfig",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let testCounter = 0;

    for (const txnType of TransactionTypes) {
      it({
        id: `T${++testCounter}`,
        title:
          `Changing FeesTreasuryProportion to zero should send 0% of fees to treasury for` +
          ` Ethereum ${txnType} transfers`,
        test: async () => {
          const param = parameterType(context, "RuntimeConfig", "FeesTreasuryProportion", 0);
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
            await createRawTransfer(context, BALTATHAR_ADDRESS, 128, { type: txnType })
          );

          const balAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
          const issuanceAfter = (
            await context.polkadotJs().query.balances.totalIssuance()
          ).toBigInt();

          const treasuryIncrease = balAfter - balBefore;
          const fee = extractFee(result?.events)!.amount.toBigInt();
          expect(treasuryIncrease, "0% of the fees should go to treasury").to.equal(0n);

          const issuanceDecrease = issuanceBefore - issuanceAfter;
          expect((fee * 100n) / issuanceDecrease, "100% of the fees should be burned").to.equal(
            100n
          );
        },
      });
    }

    it({
      id: `T${++testCounter}`,
      title:
        `Changing FeesTreasuryProportion to zero should send 0% of fees to treasury for` +
        ` Substrate based transactions with no tip`,
      test: async () => {
        const param = parameterType(context, "RuntimeConfig", "FeesTreasuryProportion", 0);
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
            .tx.balances.transferKeepAlive(alith.address, 128)
            .signAsync(baltathar, { tip: 0 }),
          { allowFailures: false }
        );

        const balanceAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
        const issuanceAfter = (
          await context.polkadotJs().query.balances.totalIssuance()
        ).toBigInt();

        const treasuryIncrease = balanceAfter - balanceBefore;
        const fee = extractFee(result?.events)!.amount.toBigInt();
        expect(treasuryIncrease, "0% of the fees should go to treasury").to.equal(0n);

        const issuanceDecrease = issuanceBefore - issuanceAfter;
        expect((fee * 100n) / issuanceDecrease, "100% of the fees should be burned").to.equal(100n);
      },
    });

    it({
      id: `T${++testCounter}`,
      title:
        `Changing FeesTreasuryProportion to zero should send 0% of fees to treasury for` +
        ` Substrate based transactions with tip`,
      test: async () => {
        const param = parameterType(context, "RuntimeConfig", "FeesTreasuryProportion", 0);
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
            .tx.balances.transferKeepAlive(alith.address, 128)
            .signAsync(baltathar, { tip: 128 }),
          { allowFailures: false }
        );

        const balanceAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
        const issuanceAfter = (
          await context.polkadotJs().query.balances.totalIssuance()
        ).toBigInt();

        const treasuryIncrease = balanceAfter - balanceBefore;
        const fee = extractFee(result?.events)!.amount.toBigInt();
        expect(treasuryIncrease, "0% of the fees should go to treasury").to.equal(0n);

        const issuanceDecrease = issuanceBefore - issuanceAfter;
        expect((fee * 100n) / issuanceDecrease, "100% of the fees should be burned").to.equal(100n);
      },
    });
  },
});
