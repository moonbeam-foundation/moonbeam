import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { MIN_GLMR_STAKING, MIN_GLMR_DELEGATOR, GLMR } from "../../util/constants";
import { DevTestContext, describeDevMoonbeam } from "../../util/setup-dev-tests";
import { alith, ethan, generateKeyringPair } from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { chunk } from "../../util/common";
import { KeyringPair } from "@substrate/txwrapper-core";
import chalk from "chalk";
import { jumpRounds } from "../../util/block";

const debug = require("debug")("test:staking-transaction-fit");

describeBenchmark("Staking - Max Transaction Fit", "joinCandidates", async (context) => {
  const maxTransactions = 100;
  const randomAccounts = await createAccounts(context, maxTransactions);
  await expectOk(
    context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.parachainStaking
          .joinCandidates(MIN_GLMR_STAKING, maxTransactions)
          .signAsync(account)
      )
    )
  );
});

describeBenchmark("Staking - Max Transaction Fit", "delegate", async (context) => {
  const maxTransactions = 350;
  const randomAccounts = await createAccounts(context, maxTransactions);
  await expectOk(
    context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.parachainStaking
          .delegate(alith.address, MIN_GLMR_DELEGATOR, maxTransactions, 0)
          .signAsync(account)
      )
    )
  );
});

describeBenchmark("Staking - Max Transaction Fit", "delegateWithAutoCompound", async (context) => {
  const maxTransactions = 350;
  const randomAccounts = await createAccounts(context, maxTransactions);
  await expectOk(
    context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.parachainStaking
          .delegateWithAutoCompound(
            alith.address,
            MIN_GLMR_DELEGATOR,
            100,
            maxTransactions,
            maxTransactions,
            0
          )
          .signAsync(account)
      )
    )
  );
});

describeBenchmark("Staking - Max Transaction Fit", "delegatorBondMore", async (context) => {
  const maxTransactions = 350;
  const randomAccounts = await createAccounts(context, maxTransactions);
  for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
    await expectOk(
      context.createBlock(
        randomAccountsChunk.map((account) =>
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              maxTransactions,
              maxTransactions,
              0
            )
            .signAsync(account)
        )
      )
    );
  }

  expect((await context.polkadotApi.query.parachainStaking.delegatorState.keys()).length).to.equal(
    maxTransactions
  );

  await expectOk(
    context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.parachainStaking
          .delegatorBondMore(alith.address, 1000)
          .signAsync(account)
      )
    )
  );
});

describeBenchmark("Staking - Max Transaction Fit", "scheduleDelegatorBondLess", async (context) => {
  const maxTransactions = 350;
  const randomAccounts = await createAccounts(context, maxTransactions);
  for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
    await expectOk(
      context.createBlock(
        randomAccountsChunk.map((account) =>
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR + 1000n,
              100,
              maxTransactions,
              maxTransactions,
              0
            )
            .signAsync(account)
        )
      )
    );
  }

  expect((await context.polkadotApi.query.parachainStaking.delegatorState.keys()).length).to.equal(
    maxTransactions
  );

  await expectOk(
    context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.parachainStaking
          .scheduleDelegatorBondLess(alith.address, 1000)
          .signAsync(account)
      )
    )
  );
});

describeBenchmark("Staking - Max Transaction Fit", "scheduleRevokeDelegation", async (context) => {
  const maxTransactions = 350;
  const randomAccounts = await createAccounts(context, maxTransactions);
  for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
    await expectOk(
      context.createBlock(
        randomAccountsChunk.map((account) =>
          context.polkadotApi.tx.parachainStaking
            .delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              100,
              maxTransactions,
              maxTransactions,
              0
            )
            .signAsync(account)
        )
      )
    );
  }

  expect((await context.polkadotApi.query.parachainStaking.delegatorState.keys()).length).to.equal(
    maxTransactions
  );

  await expectOk(
    context.createBlock(
      randomAccounts.map((account) =>
        context.polkadotApi.tx.parachainStaking
          .scheduleRevokeDelegation(alith.address)
          .signAsync(account)
      )
    )
  );
});

// utils

function describeBenchmark(
  title: string,
  method: string,
  cb: (context: DevTestContext) => Promise<void>
) {
  describeDevMoonbeam(`${title} - ${method}`, (context) => {
    it("should fit minimum 2", async function () {
      if (process.env["STAKING_BENCHMARK"] !== "1") {
        this.skip();
      }

      this.timeout(30000);
      const methodName = this.test.parent.title.split("-")[2].trim();
      await cb(context);
      const [numTransactions, weightUtil, proofUtil] = await countExtrinsics(context, methodName);
      console.log(
        `  ${chalk.yellow("○")} ${chalk.gray(methodName)} max ${chalk.green(
          numTransactions
        )} per block (w: ${(weightUtil * 100).toFixed(1)}%, p: ${(proofUtil * 100).toFixed(1)}%)`
      );
      expect(numTransactions).to.be.greaterThanOrEqual(2);
    });
  });
}

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

async function countExtrinsics(
  context: DevTestContext,
  method: string
): Promise<[number, number, number]> {
  const block = await context.polkadotApi.rpc.chain.getBlock();
  const extrinsicCount = block.block.extrinsics.reduce(
    (acc, ext) =>
      acc + (ext.method.section === "parachainStaking" && ext.method.method === method ? 1 : 0),
    0
  );

  const maxBlockWeights = context.polkadotApi.consts.system.blockWeights;
  const blockWeights = await context.polkadotApi.query.system.blockWeight();

  const weightUtil =
    blockWeights.normal.refTime.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().refTime.toNumber();
  const proofUtil =
    blockWeights.normal.proofSize.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().proofSize.toNumber();

  return [extrinsicCount, weightUtil, proofUtil];
}
