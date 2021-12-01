import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_BALANCE } from "../../util/constants";

import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Balance transfer cost", (context) => {
  it("should cost 21000 * 1_000_000_000", async function () {
    const testAccount = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAccount, 0)],
    });

    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (GENESIS_ACCOUNT_BALANCE - 21000n * 1_000_000_000n).toString()
    );
  });
});

// describeDevMoonbeam("Balance transfer cost (EIP2930)", (context) => {
//   it("should cost 21000 * 1_000_000_000", async function () {
//     const testAccount = "0x1111111111111111111111111111111111111111";
//     await context.createBlock({
//       transactions: [await createTransfer(context, testAccount, 0, { accessList: [] })],
//     });

//     expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
//       (GENESIS_ACCOUNT_BALANCE - 21000n * 1_000_000_000n).toString()
//     );
//   });
// });

// describeDevMoonbeam("Balance transfer cost (EIP1559)", (context) => {
//   it("should cost 21000 * 1_000_000_000", async function () {
//     const testAccount = "0x1111111111111111111111111111111111111111";
//     await context.createBlock({
//       transactions: [
//         await createTransfer(context, testAccount, 0, { maxFeePerGas: 1_000_000_000 }),
//       ],
//     });

//     expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
//       (GENESIS_ACCOUNT_BALANCE - 21000n * 1_000_000_000n).toString()
//     );
//   });
// });

describeDevMoonbeamAllEthTxTypes("Balance transfer", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  before("Create block with transfer to test account of 512", async () => {
    await context.createBlock({
      transactions: [await createTransfer(context, TEST_ACCOUNT, 512)],
    });
  });

  it("should decrease from account", async function () {
    // 21000 covers the cost of the transaction
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (GENESIS_ACCOUNT_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
    );
  });

  it("should increase to account", async function () {
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 0)).to.equal("0");
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal("512");
  });

  it("should reflect balance identically on polkadot/web3", async function () {
    const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (
        await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)
      ).data.free.toString()
    );
  });
});

// describeDevMoonbeam("Balance transfer (EIP2930)", (context) => {
//   const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
//   before("Create block with transfer to test account of 512", async () => {
//     await context.createBlock({
//       transactions: [await createTransfer(context, TEST_ACCOUNT, 512, { accessList: [] })],
//     });
//   });

//   it("should decrease from account", async function () {
//     // 21000 covers the cost of the transaction
//     expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
//       (GENESIS_ACCOUNT_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
//     );
//   });

//   it("should increase to account", async function () {
//     expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 0)).to.equal("0");
//     expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal("512");
//   });

//   it("should reflect balance identically on polkadot/web3", async function () {
//     const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
//     expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
//       (
//         await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)
//       ).data.free.toString()
//     );
//   });
// });

// describeDevMoonbeam("Balance transfer (EIP1559)", (context) => {
//   const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
//   before("Create block with transfer to test account of 512", async () => {
//     await context.createBlock({
//       transactions: [
//         await createTransfer(context, TEST_ACCOUNT, 512, { maxFeePerGas: 1_000_000_000 }),
//       ],
//     });
//   });

//   it("should decrease from account", async function () {
//     // 21000 covers the cost of the transaction
//     expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
//       (GENESIS_ACCOUNT_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
//     );
//   });

//   it("should increase to account", async function () {
//     expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 0)).to.equal("0");
//     expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal("512");
//   });

//   it("should reflect balance identically on polkadot/web3", async function () {
//     const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
//     expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
//       (
//         await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)
//       ).data.free.toString()
//     );
//   });
// });
