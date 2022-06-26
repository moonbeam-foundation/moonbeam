import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, generateKeyingPair } from "../../util/accounts";
import {
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  notePreimage,
} from "../../util/governance";
import { GLMR } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

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
    expect(locks[0].id.toHuman().toString()).to.be.equal("DelStake");
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

describeDevMoonbeam("Staking - Locks", (context) => {
  const randomAccount = generateKeyingPair();
  const STAKING_AMOUNT = 100n * GLMR;

  before("Setup account balance & staking", async function () {
    await context.createBlock(
      context.polkadotApi.tx.balances.transfer(randomAccount.address, STAKING_AMOUNT + 1n * GLMR)
    );
    await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, STAKING_AMOUNT, 10, 10)
        .signAsync(randomAccount)
    );

    const proposalHash = await notePreimage(
      context,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address),
      alith
    );
    await execCouncilProposal(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority(proposalHash)
    );
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 100, 1)
    );
  });

  it("should be usable for democracy vote", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.democracy
        .vote(0, {
          Standard: { balance: STAKING_AMOUNT, vote: { aye: true, conviction: 1 } },
        })
        .signAsync(randomAccount)
    );
    expect(result.successful).to.be.true;
    expect(result.events.find(({ event: { method } }) => method == "Voted")).to.not.be.undefined;
  });
});
