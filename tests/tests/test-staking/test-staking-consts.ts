import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, generateKeyringPair } from "../../util/accounts";
import { GLMR, MIN_GLMR_DELEGATOR, MIN_GLMR_STAKING } from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { KeyringPair } from "@polkadot/keyring/types";
import { expectOk } from "../../util/expect";

describeDevMoonbeam("Staking - Consts - MaxDelegationsPerDelegator", (context) => {
  const randomAccount = generateKeyringPair();
  let randomCandidates: KeyringPair[];
  let maxDelegationsPerDelegator: bigint;

  before("setup candidate & delegations upto max", async function () {
    maxDelegationsPerDelegator =
      context.polkadotApi.consts.parachainStaking.maxDelegationsPerDelegator.toBigInt();
    randomCandidates = new Array(Number(maxDelegationsPerDelegator))
      .fill(0)
      .map(() => generateKeyringPair());

    let alithNonce = await context.web3.eth.getTransactionCount(alith.address);
    await expectOk(
      context.createBlock([
        context.polkadotApi.tx.balances
          .transfer(randomAccount.address, (MIN_GLMR_DELEGATOR + GLMR) * maxDelegationsPerDelegator)
          .signAsync(alith, { nonce: alithNonce++ }),
        ...randomCandidates.map((randomCandidate) =>
          context.polkadotApi.tx.balances
            .transfer(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
            .signAsync(alith, { nonce: alithNonce++ })
        ),
      ])
    );

    await expectOk(
      context.createBlock(
        randomCandidates.map((randomCandidate) =>
          context.polkadotApi.tx.parachainStaking
            .joinCandidates(MIN_GLMR_STAKING, maxDelegationsPerDelegator)
            .signAsync(randomCandidate)
        )
      )
    );

    const candidates = await context.polkadotApi.query.parachainStaking.candidateInfo.entries();
    expect(candidates.length).to.be.equal(
      Number(maxDelegationsPerDelegator) + 1,
      "Missing candidates"
    );

    let nonce = await context.web3.eth.getTransactionCount(randomAccount.address);
    await expectOk(
      context.createBlock(
        randomCandidates.map((randomCandidate) =>
          context.polkadotApi.tx.parachainStaking
            .delegate(randomCandidate.address, MIN_GLMR_DELEGATOR, 1, maxDelegationsPerDelegator)
            .signAsync(randomAccount, { nonce: nonce++ })
        )
      )
    );
  });

  it("should fail delegation request", async function () {
    const { result } = await context.createBlock(
      context.polkadotApi.tx.parachainStaking
        .delegate(alith.address, MIN_GLMR_DELEGATOR, 1, maxDelegationsPerDelegator + 1n)
        .signAsync(randomAccount)
    );
    expect(result.successful).to.be.false;
    expect(result.error.name).to.be.equal("ExceedMaxDelegationsPerDelegator");
  });
});
