import "@moonbeam-network/api-augment";

import { u128 } from "@polkadot/types";
import {
  FrameSupportWeightsDispatchInfo,
  FrameSystemEventRecord,
  SpRuntimeDispatchError,
} from "@polkadot/types/lookup";
import { IEvent } from "@polkadot/types/types";
import { expect } from "chai";

import { alith, baltathar, ethan, generateKeyingPair } from "../../util/accounts";
import {
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  GLMR,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
} from "../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, 101n * GLMR)
    );
  });

  it("should be set when staking", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, 100n * GLMR, 10, 10)
        .signAsync(randomAccount)
    );
    const locks = await context.polkadotApi.query.balances.locks(randomAccount.address);
    expect(result.successful).to.be.true;
    expect(locks.length).to.be.equal(1, "Missing lock");
  });
});

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();

  before("Setup account balance & staking", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, 101n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, 100n * GLMR, 10, 10)
        .signAsync(randomAccount)
    );
  });

  it("should not be reusable for staking", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(baltathar.address, 100n * GLMR, 10, 10)
        .signAsync(randomAccount)
    );
    expect(result.error.name.toString()).to.be.equal("InsufficientBalance");
  });
});
