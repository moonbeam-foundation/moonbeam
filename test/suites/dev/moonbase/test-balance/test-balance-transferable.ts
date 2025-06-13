import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, GLMR, baltathar, checkBalance, generateKeyringPair } from "@moonwall/util";
import { createProposal } from "../../../../helpers/voting.ts";

describeSuite({
  id: "D020307",
  title: "Balance - Transferable",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const randomAccount = generateKeyringPair();
    const randomAddress = randomAccount.address as `0x${string}`;

    beforeAll(async function () {
      await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(randomAccount.address, 38n * GLMR),
        { allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "Consistent Transferable Balance Computation for Fee Calculation and Actual Balance",
      test: async function () {
        {
          // In this test, the following actions are performed:
          // 1. A new account is created with an initial balance of 38 GLMR.
          // 2. An identity proposal is submitted by randomAccount (~15 GLMR)
          // 3. 20 GLMR are delegated to Alith from randomAccount.
          // 4. 5 GLMR are transferred to Balthazar from randomAccount.
          // 5. A second transfer of 2 GLMR to Balthazar is performed from randomAccount.

          // Delegate to Alith
          const { result: res } = await context.createBlock(
            context
              .polkadotJs()
              .tx.parachainStaking.delegateWithAutoCompound(ALITH_ADDRESS, 20n * GLMR, 0, 10, 0, 10)
              .signAsync(randomAccount)
          );
          expect(res!.successful).to.be.true;

          // Create a proposal
          const propNum = await createProposal({ context, from: randomAccount });
          expect(propNum).toBe(0);

          // Balance after proposal
          const balanceAfter = (
            await context.polkadotJs().query.system.account(randomAccount.address)
          ).data.free.toBigInt();

          // Check the balance of randomAccount before tranfer
          const balanceBeforeTransfer = await checkBalance(context, randomAddress);
          expect(balanceBeforeTransfer).toBeGreaterThan(9n * GLMR);

          // Get fee for transfer
          const fee = await context
            .polkadotJs()
            .tx.balances.transferAllowDeath(randomAccount.address, 5n * GLMR)
            .paymentInfo(randomAccount.address);

          // Transfer 5 GLMR to Balthazar
          const { result: res3 } = await context.createBlock(
            context
              .polkadotJs()
              .tx.balances.transferAllowDeath(baltathar.address, 5n * GLMR)
              .signAsync(randomAccount)
          );

          expect(res3!.successful).to.be.true;
          expect(await checkBalance(context, randomAddress)).toBe(
            balanceBeforeTransfer - 5n * GLMR - fee.partialFee.toBigInt()
          );
          expect(await checkBalance(context, randomAddress)).toBeGreaterThan(4n * GLMR);

          // Do a second transfer of 2 GLMR to Balthazar
          const { result: res2 } = await context.createBlock(
            context
              .polkadotJs()
              .tx.balances.transferAllowDeath(baltathar.address, 2n * GLMR)
              .signAsync(randomAccount)
          );
          expect(res2!.successful).to.be.true;
        }
      },
    });
  },
});
