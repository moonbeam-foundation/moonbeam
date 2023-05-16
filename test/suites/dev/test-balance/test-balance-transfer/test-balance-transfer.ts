import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeEach, beforeAll } from "@moonwall/cli";
import { KeyringPair } from "@polkadot/keyring/types";
import {
  alith,
  ALITH_ADDRESS,
  ALITH_GENESIS_TRANSFERABLE_BALANCE,
  BALTATHAR_ADDRESS,
  generateKeyringPair,
  GLMR,
  mapExtrinsics,
  MIN_GAS_PRICE,
} from "@moonwall/util";
import { PrivateKeyAccount } from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import {
  TransactionTypes,
  checkBalance,
  createRawTransfer,
  sendRawTransaction,
} from "../../../../helpers/viem.js";

describeSuite({
  id: "D030501",
  title: "Balance Transfers",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let randomAddress: `0x${string}`;

    // beforeAll(async function () {
    //   // To create the treasury account
    //   await context.createBlock(createRawTransfer(context, BALTATHAR_ADDRESS, 1337));
    // });

    beforeEach(async function () {
      const randomAccount = generateKeyringPair();
      randomAddress = randomAccount.address as `0x${string}`;
    });

    it({
      id: "T01",
      title: "should cost 21000 gas for a transfer",
      test: async function () {
        const estimatedGas = await context.viemClient("public").estimateGas({
          account: ALITH_ADDRESS,
          value: 0n * GLMR,
          to: randomAddress,
        });
        expect(estimatedGas, "Estimated bal transfer incorrect").toBe(21000n);

        await context.createBlock(createRawTransfer(context, randomAddress, 0n));
        expect(await checkBalance(context)).toBe(
          ALITH_GENESIS_TRANSFERABLE_BALANCE - 21000n * 10_000_000_000n
        );
      },
    });

    it({
      id: "T02",
      title: "unsent txns should be in pending",
      test: async function () {
        // await context.createBlock(context.polkadotJs().tx.balances.transfer(randomAddress, 512n));
        await context.createBlock()
        const balanceBefore = await checkBalance(context, ALITH_ADDRESS, "pending");
        const rawTx = (await createRawTransfer(context, randomAddress, 512n, {
          gasPrice: MIN_GAS_PRICE,
          gas: 21000n,
          type: "legacy"
        })) as `0x${string}`;
        await sendRawTransaction(context, rawTx);
        const pendingBalance = balanceBefore - 512n - 21000n * MIN_GAS_PRICE;
        const fees = 21000n * MIN_GAS_PRICE;
        log(pendingBalance)
        log(balanceBefore)
        const balanceAfter = await checkBalance(context, ALITH_ADDRESS, "pending");
        expect(await checkBalance(context, randomAddress, "pending")).toBe(512n);
        expect(balanceAfter - balanceBefore - fees).toBe(512n);
        
      },
    });
  },
});

// privateKey,
// type: txnType,
// gasPrice: MIN_GAS_PRICE,
// gas: 21000n,
// maxFeePerGas: MIN_GAS_PRICE,

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
