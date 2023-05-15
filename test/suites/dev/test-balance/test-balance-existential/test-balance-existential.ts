import "@moonbeam-network/api-augment";
import { expect, beforeAll, describeSuite, getDevChain } from "@moonwall/cli";
import {
  alith,
  ALITH_ADDRESS,
  baltathar,
  GLMR,
  MIN_GAS_PRICE,
  RUNTIME_CONSTANTS,
} from "@moonwall/util";
import {
  PrivateKeyAccount,
  createPublicClient,
  createWalletClient,
  formatGwei,
  http,
  verifyMessage,
  webSocket,
} from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import { localhost, moonbaseAlpha, moonbeam } from "viem/chains";
import { getProviderPath, localViemNetworkDetails } from "../../../../helpers/common.js";
import Keyring from "@polkadot/keyring";
import { Wallet, ethers, Signer } from "ethers";
import { createTransfer } from "../../../../helpers/transactions.js";
// TODO: Sort out matrix tests with multi txn types, i.e. eip1550, legacy, etc.

// describeDevMoonbeamAllEthTxTypes("Existential Deposit", (context) => {
//   let randomWeb3Account: Account;
//   before("setup accounts", async function () {
//     randomWeb3Account = context.web3.eth.accounts.create("random");
//     const { result, block } = await context.createBlock(
//       createTransfer(context, randomWeb3Account.address, 10n * GLMR, {
//         from: alith.address,
//         gas: 21000,
//       })
//     );
//     expect(result.successful, result.error?.name).to.be.true;
//   });

//   it("should be disabled (no reaped account on 0 balance)", async function () {
//     const { block, result } = await context.createBlock(
//       createTransfer(context, alith.address, 10n * GLMR - 21000n * MIN_GAS_PRICE, {
//         from: randomWeb3Account.address,
//         privateKey: randomWeb3Account.privateKey,
//         gas: 21000,
//         gasPrice: MIN_GAS_PRICE,
//       })
//     );
//     expect(result.successful, result.error?.name).to.be.true;
//     expect(parseInt(await context.web3.eth.getBalance(randomWeb3Account.address))).to.eq(0);
//     expect(await context.web3.eth.getTransactionCount(randomWeb3Account.address)).to.eq(1);
//   });
// });

describeSuite({
  id: "D030101",
  title: "Existential Deposit",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let randomAccount: PrivateKeyAccount;
    let privateKey: `0x${string}`;

    beforeAll(async function () {
      privateKey = generatePrivateKey();
      randomAccount = privateKeyToAccount(privateKey);
      const { result, block } = await context.createBlock(
        context.polkadotJs().tx.balances.transferKeepAlive(randomAccount.address, 10n * GLMR)
      );
      expect(result!.successful, result!.error?.name).to.be.true;
    });

    it({
      id: "T01",
      title: "should be disabled (no reaped account on 0 balance)",
      test: async function () {
        const raw = await randomAccount.signTransaction({
          to: ALITH_ADDRESS,
          value: 10n * GLMR - 21000n * MIN_GAS_PRICE,
          chainId: await context.viemClient("public").getChainId(),
          type: "eip1559",
          gas: 21000n,
          maxFeePerGas: MIN_GAS_PRICE,
          maxPriorityFeePerGas: MIN_GAS_PRICE,
        });

        const { block, result } = await context.createBlock(raw);

        expect(
          await context.viemClient("public").getTransactionCount({ address: randomAccount.address })
        ).toBe(1);
        expect(result!.successful, result!.error?.name).toBe(true);
        expect(
          await context.viemClient("public").getBalance({ address: randomAccount.address })
        ).toBe(0n);
      },
    });
  },
});

// // run in legacy only -- this test requires that exactly its gas_price * gas_limit be deducted from
// // the sender's account
// describeDevMoonbeam("Existential Deposit", (context) => {
//   let randomWeb3Account: Account;
//   before("setup accounts", async function () {
//     randomWeb3Account = context.web3.eth.accounts.create("random");
//     await context.createBlock(
//       createTransfer(context, randomWeb3Account.address, 10n * GLMR, {
//         from: alith.address,
//         gas: 21000,
//         gasPrice: MIN_GAS_PRICE,
//       })
//     );
//   });

//   it("should be disabled (no reaped account on tiny balance - 1)", async function () {
//     await context.createBlock(
//       createTransfer(context, baltathar.address, 10n * GLMR - 1n - 21000n * MIN_GAS_PRICE, {
//         from: randomWeb3Account.address,
//         privateKey: randomWeb3Account.privateKey,
//         gas: 21000,
//         gasPrice: MIN_GAS_PRICE,
//       })
//     );
//     expect(parseInt(await context.web3.eth.getBalance(randomWeb3Account.address))).to.eq(1);
//     expect(await context.web3.eth.getTransactionCount(randomWeb3Account.address)).to.eq(1);
//   });
// });

// describeDevMoonbeam("Existential Deposit", (context) => {
//   it("checks that existantial deposit is set to zero", async function () {
//     // Grab existential deposit
//     let existentialDeposit = (await context.polkadotApi.consts.balances.existentialDeposit) as any;
//     expect(existentialDeposit.toBigInt()).to.eq(0n);
//   });
// });
