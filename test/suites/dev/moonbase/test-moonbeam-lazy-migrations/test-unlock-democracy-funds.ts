import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { votingOf, locks } from "../../../../helpers";

describeSuite({
  id: "LM01",
  title: "Lazy Migrations - Unlock Democracy Funds",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();

      await context.createBlock(
        api.tx.sudo
          .sudo(api.tx.system.setStorage(votingOf.map((v) => [v.key, v.data])))
          .signAsync(alith)
      );
      expect((await api.query.democracy.votingOf.entries()).length).is.equal(100);

      await context.createBlock(
        api.tx.sudo
          .sudo(api.tx.system.setStorage(locks.map((l) => [l.key, l.data])))
          .signAsync(alith)
      );
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
      title:
        "Complex test where we unlock all democracy funds and check that the migration state has " +
        "been changed.",
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

        /// Check that the locks are removed for the first 70 accounts
        for (let i = 0; i < 70; i++) {
          const account_locks = await api.query.balances.locks(locks[i]["account"]);
          for (let j = 0; j < account_locks.length; j++) {
            expect(account_locks[j].id.toHuman()).is.not.equal("democrac");
          }
        }

        const tx4 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(50)
        );
        expect(tx4.result?.successful, "Failed to unlock democracy funds");
        expect((await api.query.democracy.votingOf.entries()).length).is.equal(0);

        /// Check there is no democracy lock left
        for (let i = 0; i < locks.length; i++) {
          const account_locks = await api.query.balances.locks(locks[i]["account"]);
          for (let j = 0; j < account_locks.length; j++) {
            expect(account_locks[j].id.toHuman()).is.not.equal("democrac");
          }
        }

        const tx5 = await context.createBlock(
          await api.tx.moonbeamLazyMigrations.unlockDemocracyFunds(1)
        );
        expect(tx5.result?.successful).is.equal(false);
        expect(tx5.result?.error?.name).is.equal("AllDemocracyFundsUnlocked");
      },
    });
  },
});
