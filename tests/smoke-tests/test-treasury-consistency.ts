import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import type { FrameSystemAccountInfo } from "@polkadot/types/lookup";
import { expect } from "chai";
import { printTokens } from "../util/logging";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
const debug = require("debug")("smoke:treasury");

describeSmokeSuite("S2200", `Verify treasury consistency`, (context, testIt) => {
  const accounts: { [account: string]: FrameSystemAccountInfo } = {};

  let atBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise"> = null;

  before("Setup api", async function () {
    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
  });

  testIt("C100", `should have value > 0`, async function () {
    // Load data
    const treasuryPalletId = context.polkadotApi.consts.treasury.palletId;
    const treasuryAccount = await apiAt.query.system.account(
      `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`
    );

    expect(treasuryAccount.data.free.toBigInt() > 0n).to.be.true;
    expect(treasuryAccount.data.reserved.toBigInt()).to.be.equal(0n);

    debug(`Verified treasury free/reserved balance`);
  });
});
