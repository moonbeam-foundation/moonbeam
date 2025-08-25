import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { type PrivateKeyAccount, generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { VESTING_PERIOD, getAccountPayable } from "../../../../helpers";

describeSuite({
  id: "D020711",
  title: "Crowdloan - Many Accounts batch",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let numberOfAccounts: number;
    let largInput: [string, string, bigint][];

    beforeAll(async () => {
      // We shouldnt be able to register as many accounts unless we do it in batches
      numberOfAccounts = context
        .polkadotJs()
        .consts.crowdloanRewards.maxInitContributors.toNumber();
    });

    it({
      id: "T01",
      title: "should be able to register many accounts - batch",
      timeout: 20000,
      test: async function () {
        log(`${numberOfAccounts} accounts will be registered`);

        const accounts = new Array(numberOfAccounts)
          .fill(0)
          .map((_, i) => privateKeyToAccount(generatePrivateKey()));
        largInput = accounts.map((acc: PrivateKeyAccount, i: number) => {
          return [
            acc.address + "111111111111111111111111",
            acc.address,
            (3_000_000n * GLMR) / BigInt(numberOfAccounts),
          ];
        });
        expect(largInput.length).to.eq(numberOfAccounts);
        expect(largInput[0][1] !== largInput[numberOfAccounts - 1][1]).to.eq(true);

        // should be able to register many accounts
        await context.createBlock(
          context
            .polkadotJs()
            .tx.utility.batch([
              await context
                .polkadotJs()
                .tx.sudo.sudo(
                  context
                    .polkadotJs()
                    .tx.crowdloanRewards.initializeRewardVec(
                      largInput.slice(0, Math.floor(numberOfAccounts / 3))
                    )
                ),
              await context
                .polkadotJs()
                .tx.sudo.sudo(
                  context
                    .polkadotJs()
                    .tx.crowdloanRewards.initializeRewardVec(
                      largInput.slice(
                        Math.floor(numberOfAccounts / 3),
                        Math.floor((numberOfAccounts * 2) / 3)
                      )
                    )
                ),
              await context
                .polkadotJs()
                .tx.sudo.sudo(
                  context
                    .polkadotJs()
                    .tx.crowdloanRewards.initializeRewardVec(
                      largInput.slice(Math.floor((numberOfAccounts * 2) / 3), numberOfAccounts)
                    )
                ),
            ])
            .signAsync(alith)
        );

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();

        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD)
            )
        );

        await Promise.all(
          largInput.map(async (input) => {
            expect((await getAccountPayable(context, input[1]))!.totalReward.toBigInt()).to.equal(
              (3_000_000n * GLMR) / BigInt(numberOfAccounts)
            );
          })
        );
      },
    });
  },
});
