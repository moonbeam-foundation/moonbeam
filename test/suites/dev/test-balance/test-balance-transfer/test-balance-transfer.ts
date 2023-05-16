import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeEach, beforeAll } from "@moonwall/cli";
import {
  alith,
  BALTATHAR_ADDRESS,
  GLMR,
  mapExtrinsics,
} from "@moonwall/util";
import { PrivateKeyAccount } from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import { TransactionTypes, createRawTransfer } from "../../../../helpers/viem.js";

describeSuite({
  id: "D030501",
  title: "Balance - Extrinsic",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let randomAccount: PrivateKeyAccount;

    beforeAll(async function () {
      // To create the treasury account
      await context.createBlock(createRawTransfer(context, BALTATHAR_ADDRESS, 1337));
    });

    beforeEach(async function () {
      const privateKey = generatePrivateKey();
      randomAccount = privateKeyToAccount(privateKey);
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: `should emit events for ${txnType} ethereum/transfers`,
        test: async function () {
          await context.createBlock(
            createRawTransfer(context, randomAccount.address, 1n * GLMR, {
              type: txnType,
              gas: 500000n,
            })
          );

          const signedBlock = await context.polkadotJs().rpc.chain.getBlock();
          const allRecords = await context.polkadotJs().query.system.events();
          const txsWithEvents = mapExtrinsics(signedBlock.block.extrinsics, allRecords);

          const ethTx = txsWithEvents.find(
            ({ extrinsic: { method } }) => method.section == "ethereum"
          )!;

          expect(ethTx.events.length).to.eq(9);
          expect(context.polkadotJs().events.system.NewAccount.is(ethTx.events[1])).to.be.true;
          expect(context.polkadotJs().events.balances.Endowed.is(ethTx.events[2])).to.be.true;
          expect(context.polkadotJs().events.balances.Transfer.is(ethTx.events[3])).to.be.true;
          expect(ethTx.events[3].data[0].toString()).to.eq(alith.address);
          expect(ethTx.events[3].data[1].toString()).to.eq(randomAccount.address);
          expect(context.polkadotJs().events.treasury.Deposit.is(ethTx.events[6])).to.be.true;
          expect(context.polkadotJs().events.ethereum.Executed.is(ethTx.events[7])).to.be.true;
          expect(context.polkadotJs().events.system.ExtrinsicSuccess.is(ethTx.events[8])).to.be
            .true;
        },
      });
    }
  },
});


// import {
//   alith,
//   ALITH_GENESIS_LOCK_BALANCE,
//   ALITH_GENESIS_TRANSFERABLE_BALANCE,
//   baltathar,
//   generateKeyringPair,
// } from "../../util/accounts";
// import { verifyLatestBlockFees } from "../../util/block";
// import { customWeb3Request } from "../../util/providers";
// import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
// import {
//   ALITH_TRANSACTION_TEMPLATE,
//   createTransaction,
//   createTransfer,
// } from "../../util/transactions";
// import { MIN_GAS_PRICE } from "../../util/constants";

// describeDevMoonbeam("Balance transfer cost", (context) => {
//   const randomAccount = generateKeyringPair();
//   it("should cost 21000 * 10_000_000_000", async function () {
//     await context.createBlock(createTransfer(context, randomAccount.address, 0));

//     expect(await context.web3.eth.getBalance(alith.address, 1)).to.equal(
//       (ALITH_GENESIS_TRANSFERABLE_BALANCE - 21000n * 10_000_000_000n).toString()
//     );
//   });
// });

// describeDevMoonbeam("Balance transfer", (context) => {
//   const randomAccount = generateKeyringPair();
//   before("Create block with transfer to test account of 512", async () => {
//     await context.createBlock();
//     await customWeb3Request(context.web3, "eth_sendRawTransaction", [
//       await createTransfer(context, randomAccount.address, 512, { gasPrice: MIN_GAS_PRICE }),
//     ]);
//     expect(await context.web3.eth.getBalance(alith.address, "pending")).to.equal(
//       (ALITH_GENESIS_TRANSFERABLE_BALANCE - 512n - 21000n * 10_000_000_000n).toString()
//     );
//     expect(await context.web3.eth.getBalance(randomAccount.address, "pending")).to.equal("512");
//     await context.createBlock();
//   });

//   it("should decrease from account", async function () {
//     // 21000 covers the cost of the transaction
//     expect(await context.web3.eth.getBalance(alith.address, 2)).to.equal(
//       (ALITH_GENESIS_TRANSFERABLE_BALANCE - 512n - 21000n * 10_000_000_000n).toString()
//     );
//   });

//   it("should increase to account", async function () {
//     expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal("0");
//     expect(await context.web3.eth.getBalance(randomAccount.address, 2)).to.equal("512");
//   });

//   it("should reflect balance identically on polkadot/web3", async function () {
//     const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
//     expect(await context.web3.eth.getBalance(alith.address, 1)).to.equal(
//       (
//         (
//           await (await context.polkadotApi.at(block1Hash)).query.system.account(alith.address)
//         ).data.free.toBigInt() - ALITH_GENESIS_LOCK_BALANCE
//       ).toString()
//     );
//   });
// });

// describeDevMoonbeam("Balance transfer - fees", (context) => {
//   const randomAccount = generateKeyringPair();
//   before("Create block with transfer to test account of 512", async () => {
//     await context.createBlock(createTransfer(context, randomAccount.address, 512));
//   });
//   it("should check latest block fees", async function () {
//     await verifyLatestBlockFees(context, BigInt(512));
//   });
// });

// describeDevMoonbeam("Balance transfer - Multiple transfers", (context) => {
//   it("should be successful", async function () {
//     const { result } = await context.createBlock([
//       createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 0 }),
//       createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 1 }),
//       createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 2 }),
//       createTransfer(context, baltathar.address, 10n ** 18n, { nonce: 3 }),
//     ]);
//     expect(result.filter((r) => r.successful)).to.be.length(4);
//   });
// });

// describeDevMoonbeam(
//   "Balance transfer - EIP1559 fees",
//   (context) => {
//     it("should handle max_fee_per_gas", async function () {
//       const randomAccount = generateKeyringPair();
//       const preBalance = BigInt(await context.web3.eth.getBalance(alith.address));
//       // With this configuration no priority fee will be used, as the max_fee_per_gas is exactly the
//       // base fee. Expect the balances to reflect this case.
//       const maxFeePerGas = 10_000_000_000;

//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           gas: "0x5208",
//           maxFeePerGas: maxFeePerGas,
//           maxPriorityFeePerGas: "0xBEBC200", // 0.2GWEI
//           to: randomAccount.address,
//           data: "0x",
//         })
//       );
//       const postBalance = BigInt(await context.web3.eth.getBalance(alith.address));
//       const fee = BigInt(21_000 * maxFeePerGas);
//       const expectedPostBalance = preBalance - fee;

//       expect(postBalance).to.be.eq(expectedPostBalance);
//     });
//   },
//   "EIP1559"
// );

// describeDevMoonbeam(
//   "Balance transfer - EIP1559 fees",
//   (context) => {
//     it("should use partial max_priority_fee_per_gas", async function () {
//       const randomAccount = generateKeyringPair();
//       const preBalance = BigInt(await context.web3.eth.getBalance(alith.address));
//       // With this configuration only half of the priority fee will be used, as the max_fee_per_gas
//       // is 2GWEI and the base fee is 1GWEI.
//       const maxFeePerGas = 10_000_000_000 * 2;

//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           gas: "0x5208",
//           maxFeePerGas: maxFeePerGas,
//           maxPriorityFeePerGas: maxFeePerGas,
//           to: randomAccount.address,
//           data: "0x",
//         })
//       );
//       const postBalance = BigInt(await context.web3.eth.getBalance(alith.address));
//       const fee = BigInt(21_000 * maxFeePerGas);
//       const expectedPostBalance = preBalance - fee;

//       expect(postBalance).to.be.eq(expectedPostBalance);
//     });
//   },
//   "EIP1559"
// );
