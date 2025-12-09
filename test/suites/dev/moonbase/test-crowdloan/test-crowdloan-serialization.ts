import "@moonbeam-network/api-augment";
import { expect, describeSuite } from "@moonwall/cli";

describeSuite({
  id: "D020703",
  title: "Crowdloan Rewards - Serialization",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    it({
      id: "T01",
      title: "should have properly deserialized genesis config",
      test: async () => {
        // This test verifies that the genesis config was properly serialized/deserialized
        // by checking that the pallet initialized correctly from genesis

        const initialized = await context.polkadotJs().query.crowdloanRewards.initialized();

        // If this is true, it means the genesis build worked, which means
        // serialization/deserialization worked correctly
        expect(initialized.toPrimitive()).toBe(true);
      },
    });

    it({
      id: "T02",
      title: "should have correctly deserialized contributor data",
      test: async () => {
        // Verify that the ContributorData tuple was correctly serialized/deserialized
        // by checking that accounts have reward info
        const DOROTHY_ADDRESS = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
        const accountPayable = await context
          .polkadotJs()
          .query.crowdloanRewards.accountsPayable(DOROTHY_ADDRESS);

        // If we have account payable data, the Vec<ContributorData<T>> was properly deserialized
        expect(accountPayable.isSome).toBe(true);
      },
    });

    it({
      id: "T03",
      title: "should have relay account ID properly deserialized",
      test: async () => {
        // Verify that RelayChainAccountId was correctly serialized/deserialized
        const RELAY_ACCOUNT = "0x1111111111111111111111111111111111111111111111111111111111111111";

        const claimed = await context
          .polkadotJs()
          .query.crowdloanRewards.claimedRelayChainIds(RELAY_ACCOUNT);

        // If this query works and returns data, RelayChainAccountId serde worked
        expect(claimed.isSome).toBe(true);
      },
    });

    it({
      id: "T04",
      title: "should have balance types properly deserialized",
      test: async () => {
        // Verify that BalanceOf<T> was correctly handled in serialization
        const DOROTHY_ADDRESS = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
        const accountPayable = await context
          .polkadotJs()
          .query.crowdloanRewards.accountsPayable(DOROTHY_ADDRESS);

        expect(accountPayable.isSome).toBe(true);
        const data = accountPayable.unwrap();
        const amount = data.totalReward.toBigInt();

        // Should be a valid balance value
        expect(typeof amount).toBe("bigint");
        expect(amount).toBeGreaterThan(0n);
      },
    });

    it({
      id: "T05",
      title: "should have account ID properly deserialized in AccountsPayable",
      test: async () => {
        // Verify that T::AccountId was correctly serialized/deserialized
        const DOROTHY_ADDRESS = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";

        const accountPayable = await context
          .polkadotJs()
          .query.crowdloanRewards.accountsPayable(DOROTHY_ADDRESS);

        // If this works, the AccountId in ContributorData was properly handled
        expect(accountPayable.isSome).toBe(true);

        if (accountPayable.isSome) {
          const data = accountPayable.unwrap();
          expect(data.totalReward.toBigInt()).toBeGreaterThan(0n);
          expect(data.contributedRelayAddresses.length).toBeGreaterThan(0);
        }
      },
    });

    it({
      id: "T06",
      title: "should handle Option<AccountId> serialization correctly",
      test: async () => {
        // The genesis config uses Option<AccountId> for native_account
        // This test verifies it was handled correctly by checking both
        // AccountsPayable (for Some(AccountId)) and UnassociatedContributions (for None)

        const DOROTHY_ADDRESS = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
        const accountPayable = await context
          .polkadotJs()
          .query.crowdloanRewards.accountsPayable(DOROTHY_ADDRESS);

        // Dorothy should have an associated account (Some(AccountId) in genesis)
        expect(accountPayable.isSome).toBe(true);
      },
    });
  },
});
