import "@moonbeam-network/api-augment";
import { DevModeContext, afterEach, describeSuite, expect, beforeEach } from "@moonwall/cli";
import {
  GLMR,
  KeyringPair,
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  generateKeyringPair,
} from "@moonwall/util";
import chalk from "chalk";
import { Debugger } from "debug";
import { generatePrivateKey } from "viem/accounts";
import { chunk } from "../../../../tests/util/common.js";

describeSuite({
  id: "D2989",
  title: "Staking - Max Transaction Fit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "delegatorBondMore",
      timeout: 30000,
      test: async () => {
        const maxTransactions = 350;
        const randomAccounts = await createAccounts(context, maxTransactions);
        for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
          await context.createBlock(
            randomAccountsChunk.map((account) =>
              context
                .polkadotJs()
                .tx.parachainStaking.delegateWithAutoCompound(
                  alith.address,
                  MIN_GLMR_DELEGATOR,
                  100,
                  maxTransactions,
                  maxTransactions,
                  0
                )
                .signAsync(account)
            )
          );
        }

        expect(
          (await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length
        ).to.equal(maxTransactions);

        await context.createBlock(
          randomAccounts.map((account) =>
            context
              .polkadotJs()
              .tx.parachainStaking.delegatorBondMore(alith.address, 1000)
              .signAsync(account)
          )
        );

          /// Boilerplate to get the number of transactions

          const nameParts = expect.getState().currentTestName!.split(" ");
          const methodName = nameParts[nameParts.length - 1];
          const [numTransactions, weightUtil, proofUtil] = await countExtrinsics(
            context,
            methodName,
            log
          );
  
          expect(numTransactions).to.be.greaterThanOrEqual(2);

          
      },
    });
  },
});

// describeBenchmark("Staking - Max Transaction Fit", "delegatorBondMore", async (context) => {
//   const maxTransactions = 350;
//   const randomAccounts = await createAccounts(context, maxTransactions);
//   for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
//     await expectOk(
//       context.createBlock(
//         randomAccountsChunk.map((account) =>
//           context
//             .polkadotJs()
//             .tx.parachainStaking.delegateWithAutoCompound(
//               alith.address,
//               MIN_GLMR_DELEGATOR,
//               100,
//               maxTransactions,
//               maxTransactions,
//               0
//             )
//             .signAsync(account)
//         )
//       )
//     );
//   }

//   expect((await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length).to.equal(
//     maxTransactions
//   );

//   await expectOk(
//     context.createBlock(
//       randomAccounts.map((account) =>
//         context
//           .polkadotJs()
//           .tx.parachainStaking.delegatorBondMore(alith.address, 1000)
//           .signAsync(account)
//       )
//     )
//   );
// });

// describeBenchmark("Staking - Max Transaction Fit", "scheduleDelegatorBondLess", async (context) => {
//   const maxTransactions = 350;
//   const randomAccounts = await createAccounts(context, maxTransactions);
//   for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
//     await expectOk(
//       context.createBlock(
//         randomAccountsChunk.map((account) =>
//           context
//             .polkadotJs()
//             .tx.parachainStaking.delegateWithAutoCompound(
//               alith.address,
//               MIN_GLMR_DELEGATOR + 1000n,
//               100,
//               maxTransactions,
//               maxTransactions,
//               0
//             )
//             .signAsync(account)
//         )
//       )
//     );
//   }

//   expect((await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length).to.equal(
//     maxTransactions
//   );

//   await expectOk(
//     context.createBlock(
//       randomAccounts.map((account) =>
//         context
//           .polkadotJs()
//           .tx.parachainStaking.scheduleDelegatorBondLess(alith.address, 1000)
//           .signAsync(account)
//       )
//     )
//   );
// });

// describeBenchmark("Staking - Max Transaction Fit", "scheduleRevokeDelegation", async (context) => {
//   const maxTransactions = 350;
//   const randomAccounts = await createAccounts(context, maxTransactions);
//   for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
//     await expectOk(
//       context.createBlock(
//         randomAccountsChunk.map((account) =>
//           context
//             .polkadotJs()
//             .tx.parachainStaking.delegateWithAutoCompound(
//               alith.address,
//               MIN_GLMR_DELEGATOR,
//               100,
//               maxTransactions,
//               maxTransactions,
//               0
//             )
//             .signAsync(account)
//         )
//       )
//     );
//   }

//   expect((await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length).to.equal(
//     maxTransactions
//   );

//   await expectOk(
//     context.createBlock(
//       randomAccounts.map((account) =>
//         context
//           .polkadotJs()
//           .tx.parachainStaking.scheduleRevokeDelegation(alith.address)
//           .signAsync(account)
//       )
//     )
//   );
// });

// describeBenchmark("Staking - Max Transaction Fit", "scheduleLeaveDelegators", async (context) => {
//   const maxTransactions = 350;
//   const randomAccounts = await createAccounts(context, maxTransactions);
//   for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
//     await expectOk(
//       context.createBlock(
//         randomAccountsChunk.map((account) =>
//           context
//             .polkadotJs()
//             .tx.parachainStaking.delegateWithAutoCompound(
//               alith.address,
//               MIN_GLMR_DELEGATOR,
//               100,
//               maxTransactions,
//               maxTransactions,
//               0
//             )
//             .signAsync(account)
//         )
//       )
//     );
//   }

//   expect((await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length).to.equal(
//     maxTransactions
//   );

//   await expectOk(
//     context.createBlock(
//       randomAccounts.map((account) =>
//         context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(account)
//       )
//     )
//   );
// });

// describeBenchmark("Staking - Max Transaction Fit", "executeLeaveDelegators", async (context) => {
//   const maxTransactions = 350;
//   const randomAccounts = await createAccounts(context, maxTransactions);

//   await expectOk(
//     context.createBlock(
//       context
//         .polkadotJs()
//         .tx.sudo.sudo(context.polkadotJs().tx.parachainStaking.setBlocksPerRound(10))
//         .signAsync(alith)
//     )
//   );

//   for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
//     await expectOk(
//       context.createBlock(
//         randomAccountsChunk.map((account) =>
//           context
//             .polkadotJs()
//             .tx.parachainStaking.delegateWithAutoCompound(
//               alith.address,
//               MIN_GLMR_DELEGATOR,
//               100,
//               maxTransactions,
//               maxTransactions,
//               0
//             )
//             .signAsync(account)
//         )
//       )
//     );
//   }

//   expect((await context.polkadotJs().query.parachainStaking.delegatorState.keys()).length).to.equal(
//     maxTransactions
//   );

//   for (const randomAccountsChunk of chunk(randomAccounts, 20)) {
//     await expectOk(
//       context.createBlock(
//         randomAccountsChunk.map((account) =>
//           context.polkadotJs().tx.parachainStaking.scheduleLeaveDelegators().signAsync(account)
//         )
//       )
//     );
//   }

//   await jumpRounds(context, 3);

//   await expectOk(
//     context.createBlock(
//       randomAccounts.map((account) =>
//         context
//           .polkadotJs()
//           .tx.parachainStaking.executeLeaveDelegators(account.address, 1)
//           .signAsync(account)
//       )
//     )
//   );
// });

// // utils

// function describeBenchmark(
//   title: string,
//   method: string,
//   cb: (context: DevTestContext) => Promise<void>
// ) {
//   describeDevMoonbeam(`${title} - ${method}`, (context) => {
//     it("should fit minimum 2", async function () {
//       if (process.env["STAKING_BENCHMARK"] !== "1") {
//         this.skip();
//       }

//       this.timeout(30000);
//       const methodName = this.test.parent.title.split("-")[2].trim();
//       await cb(context);
//       const [numTransactions, weightUtil, proofUtil] = await countExtrinsics(context, methodName);
//       console.log(
//         `  ${chalk.yellow("○")} ${chalk.gray(methodName)} max ${chalk.green(
//           numTransactions
//         )} per block (w: ${(weightUtil * 100).toFixed(1)}%, p: ${(proofUtil * 100).toFixed(1)}%)`
//       );
//       expect(numTransactions).to.be.greaterThanOrEqual(2);
//     });
//   });
// }

async function createAccounts(
  context: DevModeContext,
  maxAccounts: number
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts))
    .fill(0)
    .map(() => generateKeyringPair());

  let alithNonce = await context
    .viem()
    .getTransactionCount({ address: alith.address as `0x${string}` });
  await context.createBlock(
    randomAccounts.map((randomCandidate) =>
      context
        .polkadotJs()
        .tx.sudo.sudo(
          context
            .polkadotJs()
            .tx.balances.setBalance(
              randomCandidate.address,
              12n * MIN_GLMR_STAKING + 50n * GLMR,
              0n
            )
        )
        .signAsync(alith, { nonce: alithNonce++ })
    ),
    { allowFailures: false }
  );

  return randomAccounts;
}

async function countExtrinsics(
  context: DevModeContext,
  method: string,
  logger: Debugger
): Promise<[number, number, number]> {
  const block = await context.polkadotJs().rpc.chain.getBlock();
  const extrinsicCount = block.block.extrinsics.reduce(
    (acc, ext) =>
      acc + (ext.method.section === "parachainStaking" && ext.method.method === method ? 1 : 0),
    0
  );

  const maxBlockWeights = context.polkadotJs().consts.system.blockWeights;
  const blockWeights = await context.polkadotJs().query.system.blockWeight();

  const weightUtil =
    blockWeights.normal.refTime.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().refTime.toNumber();
  const proofUtil =
    blockWeights.normal.proofSize.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().proofSize.toNumber();

  logger(
    `  ${chalk.yellow("○")} ${chalk.gray(method)} max ${chalk.green(
      extrinsicCount
    )} per block (w: ${(weightUtil * 100).toFixed(1)}%, p: ${(proofUtil * 100).toFixed(1)}%)`
  );

  return [extrinsicCount, weightUtil, proofUtil];
}
