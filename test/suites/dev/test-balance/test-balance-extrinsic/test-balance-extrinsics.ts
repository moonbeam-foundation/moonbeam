import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeEach } from "@moonwall/cli";
import { alith, GLMR, mapExtrinsics } from "@moonwall/util";
import { PrivateKeyAccount } from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import { TransactionTypes, createRawTransfer } from "../../../../helpers/viem.js";

describeSuite({
  id: "D030201",
  title: "Balance - Extrinsic",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let randomAccount: PrivateKeyAccount;

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
            createRawTransfer(context, randomAccount.address, 512, { type: txnType })
          );


          const blockHash = await context.polkadotJs().rpc.chain.getBlockHash(1);
              const signedBlock = await context.polkadotJs().rpc.chain.getBlock(blockHash);

              const apiAt = await context.polkadotJs().at(signedBlock.block.header.hash)
              const allRecords = await apiAt.query.system.events()
          
              const txsWithEvents = mapExtrinsics(signedBlock.block.extrinsics, allRecords);
    
            //   const ethTx = txsWithEvents.find(({extrinsic},index)=>context.polkadotJs().events.ethereum.Executed.is(extr))
          
              const ethTx = txsWithEvents.find(({ extrinsic: { method } }) => method.section == "ethereum")!;
          
            ethTx.events.forEach((event) => {
                log(event.toHuman())
            })

              expect(ethTx.events.length).to.eq(11);
              expect(context.polkadotJs().events.system.NewAccount.is(ethTx.events[1])).to.be.true;
              expect(context.polkadotJs().events.balances.Endowed.is(ethTx.events[2])).to.be.true;
              expect(context.polkadotJs().events.balances.Transfer.is(ethTx.events[3])).to.be.true;
              expect(ethTx.events[3].data[0].toString()).to.eq(alith.address);
              expect(ethTx.events[3].data[1].toString()).to.eq(randomAccount.address);
              expect(context.polkadotJs().events.balances.Endowed.is(ethTx.events[7])).to.be.true; // treasury
              expect(context.polkadotJs().events.treasury.Deposit.is(ethTx.events[8])).to.be.true;
              expect(context.polkadotJs().events.ethereum.Executed.is(ethTx.events[9])).to.be.true;
              expect(context.polkadotJs().events.system.ExtrinsicSuccess.is(ethTx.events[10])).to.be.true;

        },
      });
    }
  },
});

// describeDevMoonbeamAllEthTxTypes("Balance - Extrinsic", (context) => {
//   const randomAccount = generateKeyringPair();
//   it("should emit ethereum/transfer events", async function () {
//     await context.createBlock(createTransfer(context, randomAccount.address, 512));

//     const blockHash = await context.polkadotJs().rpc.chain.getBlockHash(1);
//     const signedBlock = await context.polkadotJs().rpc.chain.getBlock(blockHash);
//     const allRecords = (await context.polkadotJs().query.system.events.at(
//       signedBlock.block.header.hash
//     )) as any;

//     const txsWithEvents = mapExtrinsics(signedBlock.block.extrinsics, allRecords);

//     const ethTx = txsWithEvents.find(({ extrinsic: { method } }) => method.section == "ethereum");

//     expect(ethTx.events.length).to.eq(11);
//     expect(context.polkadotJs().events.system.NewAccount.is(ethTx.events[1])).to.be.true;
//     expect(context.polkadotJs().events.balances.Endowed.is(ethTx.events[2])).to.be.true;
//     expect(context.polkadotJs().events.balances.Transfer.is(ethTx.events[3])).to.be.true;
//     expect(ethTx.events[3].data[0].toString()).to.eq(alith.address);
//     expect(ethTx.events[3].data[1].toString()).to.eq(randomAccount.address);
//     expect(context.polkadotJs().events.balances.Endowed.is(ethTx.events[7])).to.be.true; // treasury
//     expect(context.polkadotJs().events.treasury.Deposit.is(ethTx.events[8])).to.be.true;
//     expect(context.polkadotJs().events.ethereum.Executed.is(ethTx.events[9])).to.be.true;
//     expect(context.polkadotJs().events.system.ExtrinsicSuccess.is(ethTx.events[10])).to.be.true;
//   });
// });
