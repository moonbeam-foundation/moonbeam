import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  DEFAULT_GENESIS_BALANCE,
  alith,
  baltathar,
  generateKeyringPair,
} from "@moonwall/util";
import { ALITH_GENESIS_TRANSFERABLE_BALANCE, verifyLatestBlockFees } from "../../../../helpers";

describeSuite({
  id: "D013701",
  title: "Sudo - successful setInflationDistributionConfig",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to call sudo with the right account",
      test: async function () {
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig(
                "ParachainBondReserve",
                {
                  account: alith.address,
                  percent: 30,
                }
              )
            )
        );
        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.inflationDistributionInfo("ParachainBondReserve");

        expect(parachainBondInfo.value.account.toString()).to.equal(alith.address);
        expect(parachainBondInfo.value.percent.toNumber()).to.equal(30);

        expect(result!.events.length).to.eq(6);
        expect(
          context
            .polkadotJs()
            .events.parachainStaking.InflationDistributionConfigUpdated.is(result!.events[1].event)
        ).to.be.true;
        expect(context.polkadotJs().events.balances.Deposit.is(result!.events[3].event)).to.be.true;
        expect(context.polkadotJs().events.system.ExtrinsicSuccess.is(result!.events[5].event)).to
          .be.true;

        expect(
          await context.viem().getBalance({ address: ALITH_ADDRESS }),
          "diff should be null for sudo - funds are sent back"
        ).to.equal(ALITH_GENESIS_TRANSFERABLE_BALANCE);
      },
    });

    it({
      id: "T02",
      title: "should charge the correct amount of gas when calling sudo",
      test: async function () {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig(
                "ParachainBondReserve",
                {
                  account: alith.address,
                  percent: 30,
                }
              )
            )
        );

        await verifyLatestBlockFees(context);
      },
    });

    it({
      id: "T03",
      title: "should NOT be able to call sudo with another account than sudo account",
      test: async function () {
        const parachainBondAccount = (
          await context.polkadotJs().query.parachainStaking.inflationDistributionInfo("ParachainBondReserve")
        ).value.account.toString();

        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.parachainStaking.setInflationDistributionConfig(
                  "ParachainBondReserve",
                  {
                    account: generateKeyringPair().address,
                    percent: 30,
                  }
                )
            )
            .signAsync(baltathar),
          { allowFailures: true }
        );

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.inflationDistributionInfo("ParachainBondReserve");
        expect(parachainBondInfo.value.account.toString()).to.equal(parachainBondAccount);
        expect(parachainBondInfo.value.percent.toNumber()).to.equal(30);

        expect(result!.events.length === 7).to.be.true;
        expect(context.polkadotJs().events.system.NewAccount.is(result!.events[2].event)).to.be
          .true;
        expect(context.polkadotJs().events.balances.Endowed.is(result!.events[3].event)).to.be.true;
        expect(context.polkadotJs().events.system.ExtrinsicFailed.is(result!.events[6].event)).to.be
          .true;

        expect(
          (await context.viem().getBalance({ address: BALTATHAR_ADDRESS })) -
          DEFAULT_GENESIS_BALANCE !==
          0n,
          "should not be null for a failed extrinsic"
        ).to.equal(true);
      },
    });

    it({
      id: "T04",
      title: "should not be able to call sudo with no funds",
      test: async function () {
        const newSigner = generateKeyringPair();
        const parachainBondAccount = (
          await context.polkadotJs().query.parachainStaking.inflationDistributionInfo("ParachainBondReserve")
        ).value.account.toString();

        await context.createBlock(context.polkadotJs().tx.sudo.setKey(newSigner.address), {
          allowFailures: false,
        });

        expect((await context.polkadotJs().query.sudo.key()).unwrap().toString()).toBe(
          newSigner.address
        );

        expect(
          async () =>
            await context.createBlock(
              context
                .polkadotJs()
                .tx.sudo.sudo(
                  context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig(
                    "ParachainBondReserve",
                    {
                      account: alith.address,
                      percent: 30,
                    }
                  )
                )
                .signAsync(newSigner)
            )
        ).rejects.toThrowError(
          "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
        );

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.inflationDistributionInfo("ParachainBondReserve");
        expect(parachainBondInfo.value.account.toString()).to.equal(parachainBondAccount);
      },
    });
  },
});
