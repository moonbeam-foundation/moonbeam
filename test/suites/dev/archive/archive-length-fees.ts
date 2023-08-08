// TODO: Split these out into moonriver/moonbeam specific suites

// describeDevMoonbeam(
//   "Substrate Length Fees - Transaction (Moonriver)",
//   (context) => {
//     it("should have low balance transfer fees", async () => {
//       const fee = await testBalanceTransfer(context);
//       expect(fee).to.equal(105268501520875n);
//     });
//   },
//   "Legacy",
//   "moonriver"
// );

// describeDevMoonbeam(
//   "Substrate Length Fees - Transaction (Moonriver)",
//   (context) => {
//     it("should have expensive runtime-upgrade fees", async () => {
//       const fee = await testRuntimeUpgrade(context);
//       expect(fee).to.equal(9_226_801_765_723_667_008n);
//     });
//   },
//   "Legacy",
//   "moonriver"
// );

// describeDevMoonbeam(
//   "Substrate Length Fees - Transaction (Moonbeam)",
//   (context) => {
//     it("should have low balance transfer fees", async () => {
//       const fee = await testBalanceTransfer(context);
//       expect(fee).to.equal(8673780152087500n);
//     });
//   },
//   "Legacy",
//   "moonbeam"
// );

// describeDevMoonbeam(
//   "Substrate Length Fees - Transaction (Moonbeam)",
//   (context) => {
//     it("should have expensive runtime-upgrade fees", async () => {
//       const fee = await testRuntimeUpgrade(context);
//       expect(fee).to.equal(922_680_166_572_366_700_800n);
//     });
//   },
//   "Legacy",
//   "moonbeam"
// );

// // define our tests here so we can DRY.
// // each test submits some txn then measures and returns the fees charged.

// const testBalanceTransfer = async (context: DevTestContext) => {
//   let initialBalance = (
//     await context.polkadotApi.query.system.account(baltathar.address)
//   ).data.free.toBigInt();

//   // send a balance transfer to self and see what our fees end up being
//   await context.createBlock(
//     context.polkadotApi.tx.balances.transfer(baltathar.address, 1).signAsync(baltathar)
//   );

//   let afterBalance = (
//     await context.polkadotApi.query.system.account(baltathar.address)
//   ).data.free.toBigInt();

//   const fee = initialBalance - afterBalance;
//   return fee;
// };

// const testRuntimeUpgrade = async (context: DevTestContext) => {
//   const initialBalance = (
//     await context.polkadotApi.query.system.account(baltathar.address)
//   ).data.free.toBigInt();

//   // generate a mock runtime upgrade hex string
//   let size = 4194304; // 2MB bytes represented in hex
//   let hex = "0x" + "F".repeat(size);

//   // send an enactAuthorizedUpgrade. we expect this to fail, but we just want to see that it was
//   // included in a block (not rejected) and was charged based on its length
//   await context.polkadotApi.tx.parachainSystem.enactAuthorizedUpgrade(hex)
//.signAndSend(baltathar);
//   await context.createBlock();

//   let afterBalance = (
//     await context.polkadotApi.query.system.account(baltathar.address)
//   ).data.free.toBigInt();

//   const fee = initialBalance - afterBalance;
//   return fee;
// };

// describeDevMoonbeam("Substrate Length Fees - Ethereum txn Interaction", (context) => {
//   it("should not charge length fee for precompile from Ethereum txn", async () => {
//     // we use modexp here because it allows us to send large-ish transactions
//     const MODEXP_PRECOMPILE_ADDRESS = "0x0000000000000000000000000000000000000005";

//     // directly call the modexp precompile with a large txn. this precompile lets us do
//two things
//     // which are useful:
//     //
//     // 1. specify an input length up to 1024 for each of mod, exp, and base
//     // 2. returns early and uses little gas (200) if all ore 0
//     //
//     // This allows us to create an Ethereum transaction whose fee is largely made up of
//Ethereum's
//     // per-byte length fee (reminder: this is 4 gas for a 0 and 16 for any non-zero byte).
//What we
//     // want to show is that this length fee is applied but our exponential LengthToFee
//(part of our
//     // Substrate-based fees) is not applied.
//     const tx = await context.web3.eth.accounts.signTransaction(
//       {
//         from: alith.address,
//         to: MODEXP_PRECOMPILE_ADDRESS,
//         gas: EXTRINSIC_GAS_LIMIT.toString(),
//         value: "0x00",
//         nonce: 0,
//         data:
//           "0x0000000000000000000000000000000000000000000000000000000000000004" + // base
//           "0000000000000000000000000000000000000000000000000000000000000004" + // exp
//           "0000000000000000000000000000000000000000000000000000000000000004" + // mod
//           "0".repeat(2048) + // 2048 hex nibbles -> 1024 bytes
//           "0".repeat(2048) +
//           "0".repeat(2048),
//       },
//       ALITH_PRIVATE_KEY
//     );

//     const result = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
//       tx.rawTransaction,
//     ]);

//     await context.createBlock();

//     const receipt = await context.web3.eth.getTransactionReceipt(result.result);
//     expect(receipt.status).to.be.true;

//     // rough math on what the exponential LengthToFee modifier would do to this:
//     // * input data alone is (3 * 1024) + (3 * 32) = 3168
//     // * 3168 ** 3 = 31_794_757_632
//     // * 31_794_757_632 / WEIGHT_PER_GAS = 1_271_790
//     //
//     // conclusion: the LengthToFee modifier is NOT involved

//     // was 33908 before Wei added the extra gas modexp cost to solve slow computation
//     const expected = 37708;
//     expect(receipt.gasUsed).to.equal(expected);

//     // furthermore, we can account for the entire fee:
//     const non_zero_byte_fee = 3 * 16;
//     const zero_byte_fee = 3165 * 4;
//     const base_ethereum_fee = 21000;
//     const modexp_min_cost = 200 * 20; // see MIN_GAS_COST in frontier's modexp precompile
//     const entire_fee = non_zero_byte_fee + zero_byte_fee + base_ethereum_fee + modexp_min_cost;
//     expect(entire_fee).to.equal(expected);
//   });
// });
