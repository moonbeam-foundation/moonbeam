import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { DEFAULT_GENESIS_STAKING, GLMR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith } from "../../util/accounts";

describeDevMoonbeam("Staking - Genesis", (context) => {
  it("should match collator locked bond", async function () {
    const locks = await context.polkadotApi.query.balances.locks(alith.address);
    const expectedLocked = DEFAULT_GENESIS_STAKING;
    expect(
      locks
        .filter((l) => l.id.toHuman() === "stkngcol")
        .reduce((p, v) => p + v.amount.toBigInt(), 0n)
        .toString(),
      `Wrong locks: \n ${locks.map((lock) => `${lock.id.toHuman()}: ${lock.amount}`).join("\n")}\n`
    ).to.equal(expectedLocked.toString());
  });

  it("should include collator from the specs", async function () {
    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    expect(collators[0].toHex().toLowerCase()).equal(alith.address.toLowerCase());
  });

  it("should have collator state as defined in the specs", async function () {
    const collator = await context.polkadotApi.query.parachainStaking.candidateInfo(alith.address);
    expect(collator.toHuman()["status"]).equal("Active");
  });

  it("should have inflation matching specs", async function () {
    const inflationInfo = await context.polkadotApi.query.parachainStaking.inflationConfig();
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
    expect(inflationInfo["expect"]["min"].toBigInt()).to.eq(100_000n * GLMR);
    expect(inflationInfo["expect"]["ideal"].toBigInt()).to.eq(200_000n * GLMR);
    expect(inflationInfo["expect"]["max"].toBigInt()).to.eq(500_000n * GLMR);
    expect(inflationInfo.toHuman()["annual"]["min"]).to.eq("4.00%");
    expect(inflationInfo.toHuman()["annual"]["ideal"]).to.eq("5.00%");
    expect(inflationInfo.toHuman()["annual"]["max"]).to.eq("5.00%");
    expect(inflationInfo.toHuman()["round"]["min"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["min"])).to.eq(8949); // 4% / blocks per year * 10^9
    expect(inflationInfo.toHuman()["round"]["ideal"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["ideal"])).to.eq(11132); // 5% / blocks per year * 10^9
    expect(inflationInfo.toHuman()["round"]["max"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["max"])).to.eq(11132); // 5% / blocks per year * 10^9
  });
});
