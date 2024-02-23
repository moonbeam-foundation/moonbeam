import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { votingOf, locks } from "./first-100-votingof-and-locks-data.json";

describeSuite({
  id: "LM01",
  title: "Lazy Migrations - Unlock Democracy Funds",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();

      for (let i = 0; i < votingOf.length; i++) {
        await context.createBlock(
          api.tx.sudo
            .sudo(api.tx.system.setStorage([[votingOf[i]["key"], votingOf[i]["data"]]]))
            .signAsync(alith)
        );
      }
      expect((await api.query.democracy.votingOf.entries()).length).is.equal(100);

      for (let i = 0; i < locks.length; i++) {
        await context.createBlock(
          api.tx.sudo
            .sudo(api.tx.system.setStorage([[locks[i]["key"], locks[i]["data"]]]))
            .signAsync(alith)
        );
      }
      // 100 just added + alith
      expect((await api.query.balances.locks.entries()).length).is.equal(101);
    });

    it({
      id: "T01",
      title: "Test unlockDemocracyFunds limit is enforced.",
      test: async function () {
        const tx1 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(51)
        );
        expect(tx1.result?.error?.name).is.equal("UnlockLimitTooHigh");
      },
    });

    it({
      id: "T02",
      title: "Test unlockDemocracyFunds drains democracy votingOf and also unlocks the funds.",
      test: async function () {
        const tx1 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(30)
        );
        expect(tx1.result?.successful, "Failed to unlock democracy funds");
        expect((await api.query.democracy.votingOf.entries()).length).is.equal(70);

        /// Check that the locks are removed for the first 30 accounts
        for (let i = 0; i < 30; i++) {
          const account_locks = await api.query.balances.locks(locks[i]["account"]);
          for (let j = 0; j < account_locks.length; j++) {
            expect(account_locks[j].id.toHuman()).is.not.equal("democrac");
          }
        }

        const tx2 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(50)
        );
        expect(tx2.result?.successful, "Failed to unlock democracy funds");
        expect((await api.query.democracy.votingOf.entries()).length).is.equal(20);

        const tx4 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(50)
        );
        expect(tx4.result?.successful, "Failed to unlock democracy funds");
        expect((await api.query.democracy.votingOf.entries()).length).is.equal(0);

        const tx5 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(1)
        );
        expect(tx5.result?.successful).is.equal(false);
        expect(tx5.result?.error?.name).is.equal("AllDemocracyFundsUnlocked");

        /// Check there is no democracy lock left
        for (let i = 0; i < locks.length; i++) {
          const account_locks = await api.query.balances.locks(locks[i]["account"]);
          for (let j = 0; j < account_locks.length; j++) {
            expect(account_locks[j].id.toHuman()).is.not.equal("democrac");
          }
        }
      },
    });
  },
});
