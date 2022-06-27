import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, generateKeyingPair } from "../../util/accounts";
import { GLMR, MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { KeyringPair } from "@polkadot/keyring/types";

const DELEGATE_AMOUNT = 100n * GLMR;

describeDevMoonbeam("Staking Consts - MaxDelegationsPerDelegator", (context) => {
  const randomAccount = generateKeyingPair();
  let randomCandidates: KeyringPair[];
  let maxDelegationsPerDelegator: bigint;
  let overMaxDelegationsPerDelegator: bigint;

  before("Setup candidate & delegations", async function () {
    this.timeout(12000);
    maxDelegationsPerDelegator =
      context.polkadotApi.consts.parachainStaking.maxDelegationsPerDelegator.toBigInt();
    overMaxDelegationsPerDelegator = maxDelegationsPerDelegator + 1n;

    randomCandidates = new Array(Number(overMaxDelegationsPerDelegator))
      .fill(0)
      .map(() => generateKeyingPair());

    let alithNonce = await context.web3.eth.getTransactionCount(alith.address);
    await context.createBlock([
      context.polkadotApi.tx.balances
        .transfer(randomAccount.address, (DELEGATE_AMOUNT + GLMR) * overMaxDelegationsPerDelegator)
        .signAsync(alith, { nonce: alithNonce++ }),
      ...randomCandidates.map((randomCandidate) =>
        context.polkadotApi.tx.balances
          .transfer(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
          .signAsync(alith, { nonce: alithNonce++ })
      ),
    ]);

    await context.createBlock(
      randomCandidates.map((randomCandidate) =>
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, overMaxDelegationsPerDelegator)
          .signAsync(randomCandidate)
      )
    );

    const candidates = await context.polkadotApi.query.parachainStaking.candidateInfo.entries();
    expect(candidates.length).to.be.equal(
      Number(overMaxDelegationsPerDelegator) + 1,
      "Missing candidates"
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await context.createBlock(
      randomCandidates.map((randomCandidate) =>
        context.polkadotApi.tx.parachainStaking
          .delegate(randomCandidate.address, DELEGATE_AMOUNT, 1, maxDelegationsPerDelegator)
          .signAsync(randomAccount, { nonce: nonce++ })
      )
    );
  });

  it("should not be exceeded", async function () {
    this.timeout(12000);
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, DELEGATE_AMOUNT, 1, maxDelegationsPerDelegator + 1n)
        .signAsync(randomAccount)
    );
    expect(result.successful).to.be.false;
    expect(result.error.name.toString()).to.be.equal("ExceedMaxDelegationsPerDelegator");
  });
});
