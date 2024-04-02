import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ZERO_ADDRESS, alith } from "@moonwall/util";

const TWENTY_PERCENT = 20;

describeSuite({
  id: "D013449",
  title: "Staking - Parachain Bond - set bond account",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be initialized at address zero",
      test: async function () {
        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
        expect(parachainBondInfo.percent.toNumber()).to.equal(30);
      },
    });

    it({
      id: "T02",
      title: "should be changeable to alith address",
      test: async function () {
        // should be able to register the genesis account for reward
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context.polkadotJs().tx.parachainStaking.setParachainBondAccount(alith.address)
            )
        );
        expect(result!.successful).to.be.true;

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.account.toString()).to.equal(alith.address);
        expect(parachainBondInfo.percent.toNumber()).to.equal(30);
      },
    });

    it({
      id: "T03",
      title: "should NOT be able set the parachain bond if NOT sudo",
      test: async function () {
        // should be able to register the genesis account for reward
        const { result } = await context.createBlock(
          context.polkadotJs().tx.parachainStaking.setParachainBondAccount(alith.address)
        );
        expect(result!.successful).to.be.false;
        expect(result!.error!.name).to.equal("BadOrigin");
      },
    });

    it({
      id: "T04",
      title: "should be able set the parachain bond reserve percent with sudo",
      test: async function () {
        // should be able to register the genesis account
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.parachainStaking.setParachainBondReservePercent(TWENTY_PERCENT)
            )
        );
        expect(result!.successful).to.be.true;

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.percent.toBigInt()).to.equal(20n);
      },
    });

    it({
      id: "T05",
      title: "should NOT be able set the parachain bond reserve percent without sudo",
      test: async function () {
        const { result } = await context.createBlock(
          context.polkadotJs().tx.parachainStaking.setParachainBondReservePercent(TWENTY_PERCENT)
        );
        expect(result!.successful).to.be.false;
        expect(result!.error!.name).to.equal("BadOrigin");
      },
    });
  },
});
