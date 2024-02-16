import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeEach, TransactionTypes } from "@moonwall/cli";
import { ALITH_ADDRESS, baltathar, GLMR, MIN_GAS_PRICE } from "@moonwall/util";
import { createRawTransfer } from "@moonwall/util";
import { Wallet } from "ethers";

describeSuite({
  id: "D0301",
  title: "Existential Deposit disabled",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    // let randomAccount: PrivateKeyAccount;
    let privateKey: `0x${string}`;
    let randomWeb3Account: any;

    beforeEach(async function () {
      // privateKey = generatePrivateKey();
      // randomAccount = privateKeyToAccount(privateKey);
      randomWeb3Account = context.web3().eth.accounts.create();
      privateKey = randomWeb3Account.privateKey;
      const { result, block } = await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(randomWeb3Account.address, 10n * GLMR)
      );
      expect(result!.successful, result!.error?.name).to.be.true;
    });

    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: `full ${txnType} transfer should not reap on 0 account balance`,
        test: async function () {
          const raw = await createRawTransfer(
            context,
            ALITH_ADDRESS,
            10n * GLMR - 21000n * MIN_GAS_PRICE,
            {
              privateKey,
              type: txnType,
              gasPrice: MIN_GAS_PRICE,
              gas: 21000n,
              maxFeePerGas: MIN_GAS_PRICE,
            }
          );
          const { result } = await context.createBlock(raw);

          expect(
            await context.viem().getTransactionCount({ address: randomWeb3Account.address })
          ).toBe(1);
          expect(result!.successful, result!.error?.name).toBe(true);
          expect(await context.viem().getBalance({ address: randomWeb3Account.address })).toBe(0n);
        },
      });
    }

    it({
      id: "T04",
      title: "should not reap on tiny balance",
      test: async function () {
        const signer = new Wallet(privateKey, context.ethers().provider);
        await signer.sendTransaction({
          to: baltathar.address,
          value: 10n * GLMR - 21000n * MIN_GAS_PRICE - 1n,
          gasPrice: MIN_GAS_PRICE,
          gasLimit: 21000n,
        });

        await context.createBlock();

        expect(await context.viem().getBalance({ address: randomWeb3Account.address })).toBe(1n);
        expect(
          await context.viem().getTransactionCount({ address: randomWeb3Account.address })
        ).toBe(1);
      },
    });

    it({
      id: "T05",
      title: "runtime constant should be set to zero",
      test: async function () {
        const existentialDeposit = context
          .polkadotJs()
          .consts.balances.existentialDeposit.toBigInt();
        expect(existentialDeposit).toBe(0n);
      },
    });
  },
});
