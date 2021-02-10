import { expect } from "chai";
import { step } from "mocha-steps";

import { describeWithMoonbeam, createAndFinalizeBlock } from "./util";
import { GLMR, GENESIS_ACCOUNT_BALANCE } from "./constants";

describeWithMoonbeam("Moonbeam RPC (Stake)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_STAKED = 1_000n * GLMR;
  step("validator bond reserved in genesis", async function () {
    const account = await context.polkadotApi.query.system.account(GENESIS_ACCOUNT);
    expect(account.data.reserved.toString()).to.equal(GENESIS_STAKED.toString());
  });

  step("validator set in genesis", async function () {
    const validators = await context.polkadotApi.query.stake.validators();
    expect((validators[0] as Buffer).toString("hex").toLowerCase()).equal(GENESIS_ACCOUNT);
  });

  it("candidates set in genesis", async function () {
    const candidates = await context.polkadotApi.query.stake.candidates(GENESIS_ACCOUNT);
    expect((candidates.toHuman() as any).id.toLowerCase()).equal(GENESIS_ACCOUNT);
  });
});
