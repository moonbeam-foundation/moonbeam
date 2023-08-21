import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, generateKeyringPair } from "../../../util/accounts";
import { expectOk } from "../../../util/expect";
import { describeDevMoonbeam, DevTestContext } from "../../../util/setup-dev-tests";
import { MIN_GLMR_STAKING, GLMR } from "../../../util/constants";
import { KeyringPair } from "@substrate/txwrapper-core";
import { chunk } from "../../../util/common";

const debug = require("debug")("test:convictionVoting");

describeDevMoonbeam("Conviction Voting - delegate", (context) => {
  it("should delegate at least 10 txs in a block", async () => {
    const randomAccounts = await createAccounts(context, 100);

    await context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.convictionVoting
          .delegate(1, alith.address, 1, 1000000000000000000n)
          .signAsync(account)
      )
    );

    const events = await context.polkadotApi.query.system.events();
    const delegatedEvents = events.reduce((acc, event) => {
      if (context.polkadotApi.events.convictionVoting.Delegated.is(event.event)) {
        acc.push({
          from: event.event.data[0].toString(),
          to: event.event.data[1].toString(),
        });
      }

      return acc;
    }, []);

    expect(delegatedEvents.length).to.be.greaterThanOrEqual(10);
  });
});

describeDevMoonbeam("Conviction Voting - undelegate", (context) => {
  let randomAccounts = [];

  before("should delegate 50 accounts", async () => {
    randomAccounts = await createAccounts(context, 50);

    for (const randomChunk of chunk(randomAccounts, 10)) {
      await expectOk(
        context.createBlock(
          randomChunk.map((account) =>
            context.polkadotApi.tx.convictionVoting
              .delegate(1, alith.address, 1, 1000000000000000000n)
              .signAsync(account)
          )
        )
      );
    }
  });

  it("should undelegate at least 10 txs in a block", async () => {
    await context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.convictionVoting.undelegate(1).signAsync(account)
      )
    );

    const events = await context.polkadotApi.query.system.events();
    const undelegatedEvents = events.reduce((acc, event) => {
      if (context.polkadotApi.events.convictionVoting.Undelegated.is(event.event)) {
        acc.push({
          who: event.event.data[0].toString(),
        });
      }

      return acc;
    }, []);

    console.log(undelegatedEvents.length);
    expect(undelegatedEvents.length).to.be.greaterThanOrEqual(10);
  });
});

async function createAccounts(
  context: DevTestContext,
  maxAccounts: number
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts)).fill(0).map(() => generateKeyringPair());

  let alithNonce = await context.web3.eth.getTransactionCount(alith.address);
  await expectOk(
    context.createBlock(
      randomAccounts.map((randomCandidate) =>
        context.polkadotApi.tx.balances
          .transfer(randomCandidate.address, MIN_GLMR_STAKING + 1n * GLMR)
          .signAsync(alith, { nonce: alithNonce++ })
      )
    )
  );

  return randomAccounts;
}
