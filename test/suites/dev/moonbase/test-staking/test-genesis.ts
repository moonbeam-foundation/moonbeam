import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { DEFAULT_GENESIS_STAKING, GLMR, alith } from "@moonwall/util";

describeSuite({
  id: "D013448",
  title: "Staking - Genesis",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should match collator locked bond",
      test: async function () {
        const locks = await context.polkadotJs().query.balances.locks(alith.address);
        const expectedLocked = DEFAULT_GENESIS_STAKING;
        expect(
          locks
            .filter((l) => l.id.toHuman() === "stkngcol")
            .reduce((p, v) => p + v.amount.toBigInt(), 0n)
            .toString(),
          `Wrong locks: \n ${locks
            .map((lock) => `${lock.id.toHuman()}: ${lock.amount}`)
            .join("\n")}\n`
        ).toBe(expectedLocked.toString());
      },
    });

    it({
      id: "T02",
      title: "should include collator from the specs",
      test: async function () {
        const collators = await context.polkadotJs().query.parachainStaking.selectedCandidates();
        expect(collators[0].toHex().toLowerCase()).equal(alith.address.toLowerCase());
      },
    });

    it({
      id: "T03",
      title: "should have collator state as defined in the specs",
      test: async function () {
        const collator = await context
          .polkadotJs()
          .query.parachainStaking.candidateInfo(alith.address);
        expect(collator.unwrap().status.isActive).toBe(true);
      },
    });

    it({
      id: "T04",
      title: "should have inflation matching specs",
      test: async function () {
        const inflationInfo = await context.polkadotJs().query.parachainStaking.inflationConfig();
        // {
        //   expect: {
        //     min: '100.0000 kUNIT',
        //     ideal: '200.0000 kUNIT',
        //     max: '500.0000 kUNIT'
        //   },
        //  annual: {
        //     min: '4.00%',
        //     ideal: '5.00%',
        //     max: '5.00%',
        // },
        //   round: { min: '0.00%', ideal: '0.00%', max: '0.00%' }
        // }

        // Percentages are in perbill (1/1,000,000,000)
        expect(inflationInfo.expect.min.toBigInt()).toBe(100_000n * GLMR);
        expect(inflationInfo.expect.ideal.toBigInt()).toBe(200_000n * GLMR);
        expect(inflationInfo.expect.max.toBigInt()).toBe(500_000n * GLMR);
        expect(inflationInfo.annual.min.toBigInt()).toBe(40_000_000n);
        expect(inflationInfo.annual.ideal.toBigInt()).toBe(50_000_000n);
        expect(inflationInfo.annual.max.toBigInt()).toBe(50_000_000n);
        expect(inflationInfo.round.min.toBigInt()).toBe(8949n); // 4% / blocks per year * 10^9
        expect(inflationInfo.round.ideal.toBigInt()).toBe(11132n); // 5% / blocks per year * 10^9
        expect(inflationInfo.round.max.toBigInt()).toBe(11132n); // 5% / blocks per year * 10^9
      },
    });
  },
});
