import "@moonbeam-network/api-augment";
import { type DevModeContext, beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  GLMR,
  type KeyringPair,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";
import { chunk } from "../../../../helpers";

describeSuite({
  id: "D010707",
  title: "Conviction Voting - undelegate",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let randomAccounts: KeyringPair[];

    beforeAll(async () => {
      randomAccounts = await createAccounts(context, 50);

      for (const randomChunk of chunk(randomAccounts, 10)) {
        await context.createBlock(
          randomChunk.map((account) =>
            context
              .polkadotJs()
              .tx.convictionVoting.delegate(1, alith.address, 1, 1000000000000000000n)
              .signAsync(account)
          ),
          { allowFailures: false }
        );
      }
    });

    it({
      id: "T01",
      title: "should undelegate at least 10 txs in a block",
      test: async () => {
        await context.createBlock(
          randomAccounts.map((account) =>
            context.polkadotJs().tx.convictionVoting.undelegate(1).signAsync(account)
          )
        );

        const events = await context.polkadotJs().query.system.events();
        const undelegatedEvents = events.reduce(
          (acc, event) => {
            if (context.polkadotJs().events.convictionVoting.Undelegated.is(event.event)) {
              acc.push({
                who: event.event.data[0].toString(),
              });
            }

            return acc;
          },
          [] as { who: string }[]
        );

        console.log(undelegatedEvents.length);
        expect(undelegatedEvents.length).to.be.greaterThanOrEqual(10);
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
