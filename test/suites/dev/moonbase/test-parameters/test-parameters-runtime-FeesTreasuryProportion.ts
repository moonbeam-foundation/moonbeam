import { describeSuite, expect, extractInfo, TransactionTypes } from "@moonwall/cli";
import {
  alith,
  ALITH_ADDRESS,
  baltathar,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  createRawTransfer,
  extractFee,
  Perbill,
  TREASURY_ACCOUNT,
  WEIGHT_PER_GAS,
} from "@moonwall/util";
import { parameterType, UNIT } from "./test-parameters";
import { BN } from "@polkadot/util";
import { calculateFeePortions, ConstantStore, verifyLatestBlockFees } from "../../../../helpers";
import { parseGwei } from "viem";

interface TestCase {
  proportion: Perbill;

  transfer_amount: bigint;
  tipAmount: bigint;
  priorityFeePerGas: bigint;
}

describeSuite({
  id: "DTemp03",
  title: "Parameters - RuntimeConfig",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let testCounter = 0;
    const collatorAddress = ALITH_ADDRESS;
    const senderPrivateKey = BALTATHAR_PRIVATE_KEY;
    const senderKeyPair = baltathar;
    const receiver = CHARLETH_ADDRESS;

    const testCases: TestCase[] = [
      {
        proportion: new Perbill(0),
        transfer_amount: 10n * UNIT,
        tipAmount: 1n * UNIT,
        priorityFeePerGas: parseGwei("1"),
      },
      {
        proportion: new Perbill(1, 100),
        transfer_amount: 1000n,
        tipAmount: 100n,
        priorityFeePerGas: 100n,
      },
      {
        proportion: new Perbill(355, 1000),
        transfer_amount: 5n * UNIT,
        tipAmount: 111112n,
        priorityFeePerGas: 111112n,
      },
      {
        proportion: new Perbill(400, 1000),
        transfer_amount: 10n * UNIT,
        tipAmount: 2n * UNIT,
        priorityFeePerGas: parseGwei("2"),
      },
      {
        proportion: new Perbill(500, 1000),
        transfer_amount: 10n * UNIT,
        tipAmount: 1n * UNIT,
        priorityFeePerGas: parseGwei("1"),
      },
      {
        proportion: new Perbill(963, 1000),
        transfer_amount: 10n * UNIT,
        tipAmount: 128n,
        priorityFeePerGas: 128,
      },
      {
        proportion: new Perbill(99, 100),
        transfer_amount: 10n * UNIT,
        tipAmount: 3n,
        priorityFeePerGas: 3n,
      },
      {
        proportion: new Perbill(1, 1),
        transfer_amount: 10n * UNIT,
        tipAmount: 32n,
        priorityFeePerGas: 32n,
      },
    ];

    for (const t of testCases) {
      const treasuryPerbill = new BN(t.proportion.value());
      const treasuryPercentage = t.proportion.value().toNumber() / 1e7;
      const burnPercentage = 100 - treasuryPercentage;

      const calcTreasuryIncrease = (feeWithTip: bigint, tip?: bigint): bigint => {
        const issuanceDecrease = calcIssuanceDecrease(feeWithTip, tip);
        const treasuryIncrease = feeWithTip - issuanceDecrease;
        return treasuryIncrease;
      };
      const calcIssuanceDecrease = (feeWithTip: bigint, maybeTip?: bigint): bigint => {
        const tip = maybeTip ?? 0n;
        const feeWithoutTip = feeWithTip - tip;
        const { burnt: feeBurnt } = calculateFeePortions(treasuryPerbill, feeWithoutTip);
        const { burnt: tipBurnt } = calculateFeePortions(treasuryPerbill, tip);

        return feeBurnt + tipBurnt;
      };

      for (const txnType of TransactionTypes) {
        for (const withTip of txnType === "eip1559" ? [false, true] : [false]) {
          testCounter++;
          it({
            id: `T${testCounter}x`,
            title:
              `Changing FeesTreasuryProportion to ${treasuryPercentage}% for Ethereum ` +
              `${txnType} transfers with ${withTip ? "" : "no "}tip`,
            test: async () => {
              const { specVersion } = context.polkadotJs().consts.system.version;
              const GENESIS_BASE_FEE = ConstantStore(context).GENESIS_BASE_FEE.get(
                specVersion.toNumber()
              );
              const WEIGHT_FEE = ConstantStore(context).WEIGHT_FEE.get(specVersion.toNumber());

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
              const collatorBalBefore = await context
                .viem()
                .getBalance({ address: collatorAddress });
              const issuanceBefore = (
                await context.polkadotJs().query.balances.totalIssuance()
              ).toBigInt();

              const nextFeeMultiplier = (
                await context.polkadotJs().query.transactionPayment.nextFeeMultiplier()
              ).toBigInt();
              const baseFee =
                (nextFeeMultiplier * (WEIGHT_FEE * WEIGHT_PER_GAS)) / 1_000_000_000_000_000_000n;

              const { result } = await context.createBlock(
                await createRawTransfer(context, receiver, t.transfer_amount, {
                  privateKey: senderPrivateKey,
                  type: txnType,
                  maxFeePerGas: withTip ? GENESIS_BASE_FEE : undefined,
                  maxPriorityFeePerGas: withTip ? t.priorityFeePerGas : undefined,
                })
              );

              const receipt = await context
                .viem("public")
                .getTransactionReceipt({ hash: result!.hash as `0x${string}` });

              const balAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
              const collatorBalAfter = await context
                .viem()
                .getBalance({ address: collatorAddress });
              const issuanceAfter = (
                await context.polkadotJs().query.balances.totalIssuance()
              ).toBigInt();

              const treasuryIncrease = balAfter - balBefore;
              const issuanceDecrease = issuanceBefore - issuanceAfter;
              const collatorIncrease = collatorBalAfter - collatorBalBefore;
              const tipPaid = withTip
                ? (() => {
                    let priorityPerGas = GENESIS_BASE_FEE - baseFee;
                    if (priorityPerGas > t.priorityFeePerGas) {
                      priorityPerGas = t.priorityFeePerGas;
                    }
                    return BigInt(priorityPerGas) * BigInt(receipt!.gasUsed);
                  })()
                : 0n;
              const fee = extractFee(result?.events)!.amount.toBigInt();
              const feeWithoutTip = fee - tipPaid;

              expect(
                treasuryIncrease + issuanceDecrease,
                `Sum of TreasuryIncrease and IssuanceDecrease should be equal to the fees without tip`
              ).to.equal(feeWithoutTip);

              expect(
                treasuryIncrease,
                `${treasuryPercentage}% of the fees should go to treasury`
              ).to.equal(calcTreasuryIncrease(feeWithoutTip));

              expect(issuanceDecrease, `${burnPercentage}% of the fees should be burned`).to.equal(
                calcIssuanceDecrease(feeWithoutTip)
              );

              if (withTip) {
                expect(collatorIncrease, "100% of the tip should go to the collator").to.equal(
                  tipPaid
                );
              } else {
                expect(collatorIncrease, "No tip should be paid to the collator").to.equal(0n);
              }

              await verifyLatestBlockFees(context, t.transfer_amount);
            },
          });
        }
      }

      for (const withTip of [false, true]) {
        testCounter++;
        it({
          id: `T${testCounter}x`,
          title:
            `Changing FeesTreasuryProportion to ${treasuryPercentage}% for Substrate based ` +
            `transactions with ${withTip ? "" : "no "}tip`,
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
            const collatorBalBefore = await context.viem().getBalance({ address: collatorAddress });

            const { result } = await context.createBlock(
              context
                .polkadotJs()
                .tx.balances.transferKeepAlive(receiver, t.transfer_amount)
                .signAsync(senderKeyPair, { tip: withTip ? t.tipAmount : 0n }),
              { allowFailures: false }
            );

            const balanceAfter = await context.viem().getBalance({ address: TREASURY_ACCOUNT });
            const issuanceAfter = (
              await context.polkadotJs().query.balances.totalIssuance()
            ).toBigInt();
            const collatorBalAfter = await context.viem().getBalance({ address: collatorAddress });

            const treasuryIncrease = balanceAfter - balanceBefore;
            const issuanceDecrease = issuanceBefore - issuanceAfter;
            const collatorIncrease = collatorBalAfter - collatorBalBefore;
            const tipPaid = withTip ? t.tipAmount : 0n;
            const feeWithoutTip = extractFee(result?.events)!.amount.toBigInt() - tipPaid;

            expect(
              treasuryIncrease + issuanceDecrease,
              `Sum of TreasuryIncrease and IssuanceDecrease should be equal to the fees without tip`
            ).to.equal(feeWithoutTip);

            expect(
              treasuryIncrease,
              `${treasuryPercentage}% of the fees should go to treasury`
            ).to.equal(calcTreasuryIncrease(feeWithoutTip));

            expect(issuanceDecrease, `${burnPercentage}% of the fees should be burned`).to.equal(
              calcIssuanceDecrease(feeWithoutTip)
            );

            if (withTip) {
              expect(collatorIncrease, "100% of the tip should go to the collator").to.equal(
                t.tipAmount
              );
            } else {
              expect(collatorIncrease, "No tip should be paid to the collator").to.equal(0n);
            }

            await verifyLatestBlockFees(context, t.transfer_amount);
          },
        });
      }
    }
  },
});
