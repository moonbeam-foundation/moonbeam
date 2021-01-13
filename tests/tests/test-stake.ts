import { expect } from "chai";
import { step } from "mocha-steps";

import { describeWithMoonbeam } from "./util";

describeWithMoonbeam("Moonbeam RPC (Stake)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_STAKED = "100000";
  step("validator bond reserved in genesis", async function () {
    const account = await context.polkadotApi.query.system.account(GENESIS_ACCOUNT);
    expect(account.data.reserved.toString()).to.equal(GENESIS_STAKED);
  });
  step("validator set in genesis", async function () {
    const validators = await context.polkadotApi.query.stake.validators();
    expect((validators[0] as Buffer).toString("hex").toLowerCase()).equal(GENESIS_ACCOUNT);
  });
});
