import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeEach, beforeAll, TransactionTypes } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, GLMR, mapExtrinsics } from "@moonwall/util";
import { PrivateKeyAccount } from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import { createRawTransfer } from "@moonwall/util";

describeSuite({
  id: "D0302",
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
          expect(ethTx.events[3].data[0].toString()).to.eq(ALITH_ADDRESS);
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
