import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { MIN_GLMR_DELEGATOR, alith, ethan } from "@moonwall/util";

describeSuite({
  id: "D023418",
  title: "Staking - Delegate With Auto-Compound - valid request",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    const numberToHex = (n: bigint): string => `0x${n.toString(16).padStart(32, "0")}`;
    let events: any[];

    beforeAll(async () => {
      const { result } = await context.createBlock(
        context
          .polkadotJs()
          .tx.parachainStaking.delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            50,
            0,
            0,
            0
          )
          .signAsync(ethan)
      );
      expect(result!.successful).to.be.true;
      events = result!.events;
    });

    it({
      id: "T01",
      title: "should succeed",
      test: async () => {
        const delegatorState = await context
          .polkadotJs()
          .query.parachainStaking.delegatorState(ethan.address);
        const autoCompoundConfig = (
          (await context
            .polkadotJs()
            .query.parachainStaking.autoCompoundingDelegations(alith.address)) as any
        )
          .toJSON()
          .find((d: any) => d.delegator === ethan.address);
        const delegationEvents = events.reduce((acc, event) => {
          if (context.polkadotJs().events.parachainStaking.Delegation.is(event.event)) {
            acc.push({
              account: event.event.data[0].toString(),
              amount: event.event.data[1].toBigInt(),
              autoCompound: event.event.data[4].toJSON(),
            });
          }
          return acc;
        }, []);

        expect(delegationEvents).to.deep.equal([
          {
            account: ethan.address,
            amount: 1000000000000000000n,
            autoCompound: 50,
          },
        ]);
        expect(delegatorState.unwrap().toJSON()).to.deep.equal({
          delegations: [
            {
              amount: numberToHex(MIN_GLMR_DELEGATOR),
              owner: alith.address,
            },
          ],
          id: ethan.address,
          lessTotal: 0,
          status: { active: null },
          total: numberToHex(MIN_GLMR_DELEGATOR),
        });
        expect(autoCompoundConfig).to.deep.equal({
          delegator: ethan.address,
          value: 50,
        });
      },
    });
  },
});
