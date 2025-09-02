import "@moonbeam-network/api-augment";
import { type DevModeContext, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  GLMR,
  type KeyringPair,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";

describeSuite({
  id: "D020606",
  title: "Conviction Voting - delegate",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should delegate at least 10 txs in a block",
      test: async () => {
        const randomAccounts = await createAccounts(context, 100);

        await context.createBlock(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.convictionVoting.delegate(1, alith.address, 1, 1000000000000000000n)
              .signAsync(account)
          )
        );

        const events = await context.polkadotJs().query.system.events();
        const delegatedEvents = events.reduce(
          (acc, event) => {
            if (context.polkadotJs().events.convictionVoting.Delegated.is(event.event)) {
              acc.push({
                from: event.event.data[0].toString(),
                to: event.event.data[1].toString(),
              });
            }

            return acc;
          },
          [] as { from: string; to: string }[]
        );

        expect(delegatedEvents.length).to.be.greaterThanOrEqual(10);
      },
    });
  },
});

async function createAccounts(
  context: DevModeContext,
  maxAccounts: number
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts)).fill(0).map(() => generateKeyringPair());

  let alithNonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });
  await context.createBlock(
    randomAccounts.map((randomCandidate) =>
      context
        .polkadotJs()
        .tx.balances.transferAllowDeath(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
        .signAsync(alith, { nonce: alithNonce++ })
    ),
    { allowFailures: false }
  );

  return randomAccounts;
}
