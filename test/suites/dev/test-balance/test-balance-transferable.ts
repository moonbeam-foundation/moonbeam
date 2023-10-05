import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  GLMR,
  MIN_GLMR_DELEGATOR,
  baltathar,
  checkBalance,
  generateKeyringPair,
} from "@moonwall/util";
import { blake2AsHex } from "@polkadot/util-crypto";

describeSuite({
  id: "D4005",
  title: "Balance - Transferable",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let randomAddress: `0x${string}`;
    const randomAccount = generateKeyringPair();
    randomAddress = randomAccount.address as `0x${string}`;

    beforeAll(async function () {
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(randomAccount.address, 20n * GLMR),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "Consistent Transferable Balance Computation for Fee Calculation and Actual Balance",
      test: async function () {
        {
          // In this test, the following actions are performed:
          // 1. A new account is created with an initial balance of 20 GLMR.
          // 2. A proposal is submitted by randomAccount with a deposit of `minDepositAmount`
          //    (4 GLMR).
          // 3. 6 GLMR are delegated to Alith from randomAccount.
          // 4. 10 GLMR is transferred to Balthazar from randomAccount.

          // Retrieve the minimum deposit amount
          const minDepositAmount = context.polkadotJs().consts.democracy.minimumDeposit.toBigInt();

          // Create a proposal
          const proposal = context.polkadotJs().tx.balances.forceSetBalance(baltathar.address, 100);
          const encodedProposal = proposal.method.toHex();
          const encodedHash = blake2AsHex(encodedProposal);

          await context.createBlock(
            context.polkadotJs().tx.preimage.notePreimage(encodedProposal).signAsync(randomAccount)
          );

          await context.createBlock(
            context
              .polkadotJs()
              .tx.democracy.propose(
                {
                  Lookup: {
                    hash: encodedHash,
                    len: proposal.method.encodedLength,
                  },
                },
                minDepositAmount
              )
              .signAsync(randomAccount)
          );

          // Delegate to Alith
          await context.createBlock(
            context
              .polkadotJs()
              .tx.parachainStaking.delegate(ALITH_ADDRESS, 6n * GLMR, 10, 10)
              .signAsync(randomAccount)
          );

          // Check the balance of randomAccount before tranfer
          const balanceBeforeTransfer = await checkBalance(context, randomAddress);
          expect(balanceBeforeTransfer).toBeGreaterThan(10n * GLMR);

          // Get fee for transfer
          const fee = await context
            .polkadotJs()
            .tx.balances.transfer(randomAccount.address, 10n * GLMR)
            .paymentInfo(randomAccount.address);

          // Transfer 10 GLMR to Balthazar
          const { result } = await context.createBlock(
            context
              .polkadotJs()
              .tx.balances.transfer(baltathar.address, 10n * GLMR)
              .signAsync(randomAccount)
          );

          expect(result!.successful).to.be.true;
          expect(await checkBalance(context, randomAddress)).toBe(
            balanceBeforeTransfer - 10n * GLMR - fee.partialFee.toBigInt()
          );
        }
      },
    });
  },
});
