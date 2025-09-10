import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { type PrivateKeyAccount, generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { VESTING_PERIOD, getAccountPayable } from "../../../../helpers";

describeSuite({
  id: "D020710",
  title: "Crowdloan - many accounts",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let numberOfAccounts = -1;
    let largInput: [string, string, bigint][];

    beforeAll(async () => {
      numberOfAccounts = context
        .polkadotJs()
        .consts.crowdloanRewards.maxInitContributors.toNumber();
    });

    it({
      id: "T01",
      title: "should be able to register many accounts",
      timeout: 30000,
      test: async function () {
        log(`${numberOfAccounts} accounts will be registered`);
        // should create a bunch of test eth accounts
        // We need to make sure the rewards match the account funds. 3M GLMR/ number of accounts
        const accounts = new Array(numberOfAccounts)
          .fill(0)
          .map((_) => privateKeyToAccount(generatePrivateKey()));
        largInput = accounts.map((acc: PrivateKeyAccount) => {
          return [
            `${acc.address}111111111111111111111111`,
            acc.address,
            (3_000_000n * GLMR) / BigInt(numberOfAccounts),
          ];
        });
        expect(largInput.length).to.eq(numberOfAccounts);
        expect(largInput[0][1] !== largInput[numberOfAccounts - 1][1]).to.eq(true);

        // should be able to register many accounts
        await context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.crowdloanRewards.initializeRewardVec(largInput))
          .signAndSend(alith);

        await context.createBlock();
        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();

        // Complete initialization
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
            )
        );

        const rewardPerContributor = (3_000_000n * GLMR) / BigInt(numberOfAccounts);
        await Promise.all(
          largInput.map(async (input) => {
            expect((await getAccountPayable(context, input[1]))!.totalReward.toBigInt()).to.equal(
              rewardPerContributor
            );
          })
        );
      },
    });
  },
});
