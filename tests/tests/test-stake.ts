import { expect } from "chai";
import { step } from "mocha-steps";

import { describeWithMoonbeam } from "./util";
import { GLMR } from "./constants";

describeWithMoonbeam("Moonbeam RPC (Stake)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_STAKED = 1_000n * GLMR;
  step("collator bond reserved in genesis", async function () {
    const account = await context.polkadotApi.query.system.account(GENESIS_ACCOUNT);
    expect(account.data.reserved.toString()).to.equal(GENESIS_STAKED.toString());
  });

  step("collator set in genesis", async function () {
    const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    expect((collators[0] as Buffer).toString("hex").toLowerCase()).equal(GENESIS_ACCOUNT);
  });

  it("candidates set in genesis", async function () {
    const candidates = await context.polkadotApi.query.parachainStaking.collatorState(
      GENESIS_ACCOUNT
    );
    expect(candidates.toHuman()["id"].toLowerCase()).equal(GENESIS_ACCOUNT);
    expect(candidates.toHuman()["state"]).equal("Active");
  });

  it("inflation set in genesis", async function () {
    const inflationInfo = await context.polkadotApi.query.parachainStaking.inflationConfig();
    // {
    //   expect: {
    //     min: '100.0000 kUnit',
    //     ideal: '200.0000 kUnit',
    //     max: '500.0000 kUnit'
    //   },
    //   round: { min: '0.00%', ideal: '0.00%', max: '0.00%' }
    // }
    expect(inflationInfo.toHuman()["expect"]["min"]).to.eq("100.0000 kUnit");
    expect(inflationInfo.toHuman()["expect"]["ideal"]).to.eq("200.0000 kUnit");
    expect(inflationInfo.toHuman()["expect"]["max"]).to.eq("500.0000 kUnit");
    expect(inflationInfo.toHuman()["round"]["min"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["min"])).to.eq(4563); // 4% / 8766 * 10^9
    expect(inflationInfo.toHuman()["round"]["ideal"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["ideal"])).to.eq(5703); // 5% / 8766 * 10^9
    expect(inflationInfo.toHuman()["round"]["max"]).to.eq("0.00%");
    expect(Number(inflationInfo["round"]["max"])).to.eq(5703); // 5% / 8766 * 10^9
  });
});
