import "@moonbeam-network/api-augment";
import { ZERO_ADDRESS, charleth, describeSuite, dorothy, expect } from "moonwall";

const TWENTY_PERCENT = 20;

describeSuite({
  id: "D023350",
  title: "Staking - setInflationDistributionConfig",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should be initialized at address zero",
      test: async function () {
        const inflationDistributionConfig = await context
          .polkadotJs()
          .query.parachainStaking.inflationDistributionInfo();
        expect(inflationDistributionConfig[0].account.toString()).to.equal(ZERO_ADDRESS);
        expect(inflationDistributionConfig[0].percent.toNumber()).to.equal(30);

        // Treasury account
        expect(inflationDistributionConfig[1].account.toString()).to.equal(ZERO_ADDRESS);
        expect(inflationDistributionConfig[1].percent.toNumber()).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "should be able set the parachain bond reserve percent with sudo",
      test: async function () {
        // should be able to register the genesis account
        const { result } = await context.createBlock(
          context.polkadotJs().tx.sudo.sudo(
            context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig([
              {
                account: dorothy.address,
                percent: TWENTY_PERCENT,
              },
              {
                account: charleth.address,
                percent: TWENTY_PERCENT,
              },
            ])
          )
        );
        expect(result!.successful).to.be.true;

        const inflationDistributionConfig = await context
          .polkadotJs()
          .query.parachainStaking.inflationDistributionInfo();
        expect(inflationDistributionConfig[0].percent.toBigInt()).to.equal(20n);
      },
    });

    it({
      id: "T03",
      title: "should NOT be able set the parachain bond reserve percent without sudo",
      test: async function () {
        const { result } = await context.createBlock(
          context.polkadotJs().tx.parachainStaking.setInflationDistributionConfig([
            {
              account: dorothy.address,
              percent: TWENTY_PERCENT,
            },
            {
              account: charleth.address,
              percent: TWENTY_PERCENT,
            },
          ])
        );
        expect(result!.successful).to.be.false;
        expect(result!.error!.name).to.equal("BadOrigin");
      },
    });
  },
});
