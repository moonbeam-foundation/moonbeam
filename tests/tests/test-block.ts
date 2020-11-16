import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

import {SignedTransaction, TransactionConfig} from 'web3-core'

describeWithMoonbeam("Moonbeam RPC (Block)", `simple-specs.json`, (context) => {
  let previousBlock;
  // Those tests are dependant of each other in the given order.
  // The reason is to avoid having to restart the node each time
  // Running them individually will result in failure

  step("should be at block 0 at genesis", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(0);
  });

  it("should return genesis block", async function () {
    expect(await context.web3.eth.getBlockNumber()).to.equal(0);
    const block = await context.web3.eth.getBlock(0);
    expect(block).to.include({
      author: "0x0000000000000000000000000000000000000000",
      difficulty: "0",
      extraData: "0x",
      gasLimit: 0,
      gasUsed: 0,
      //hash: "0x14fe6f7c93597f79b901f8b5d7a84277a90915b8d355959b587e18de34f1dc17",
      logsBloom: `0x${"0".repeat(512)}`,
      number: 0,
      //parentHash: "0x2cc74f91423ba20e9bb0b2c7d8924eacd14bc98aa1daad078f8844e529221cde",
      receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      // size: 533,
      stateRoot: "0x0000000000000000000000000000000000000000000000000000000000000000",
      //timestamp: 1595012243836,
      totalDifficulty: null,
      //transactions: [],
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      //uncles: []
    });

    expect(block.transactions).to.be.a("array").empty;
    expect(block.uncles).to.be.a("array").empty;
    expect((block as any).sealFields).to.eql([
      "0x0000000000000000000000000000000000000000000000000000000000000000",
      "0x0000000000000000",
    ]);
    expect(block.hash).to.be.a("string").lengthOf(66);
    expect(block.parentHash).to.be.a("string").lengthOf(66);
    expect(block.timestamp).to.be.a("number");
  });

  let firstBlockCreated = false;
  it("should be at block 1 after block production", async function () {
    this.timeout(15000);
    await createAndFinalizeBlock(context.web3);
    expect(await context.web3.eth.getBlockNumber()).to.equal(1);
    firstBlockCreated = true;
  });

  it("should have valid timestamp after block production", async function () {
    // Originally ,this test required the timestamp be in the last finve minutes.
    // This requirement doesn't make sense when we forge timestamps in manual seal.
    const block = await context.web3.eth.getBlock("latest");
    const next5Minutes = Date.now() / 1000 + 300;
    expect(block.timestamp).to.be.least(0);
    expect(block.timestamp).to.be.below(next5Minutes);
  });

  it("retrieve block information", async function () {
    expect(firstBlockCreated).to.be.true;

    const block = await context.web3.eth.getBlock("latest");
    expect(block).to.include({
      author: "0x0000000000000000000000000000000000000000",
      difficulty: "0",
      extraData: "0x",
      gasLimit: 0,
      gasUsed: 0,
      //hash: "0x14fe6f7c93597f79b901f8b5d7a84277a90915b8d355959b587e18de34f1dc17",
      logsBloom: `0x${"0".repeat(512)}`,
      miner: "0x0000000000000000000000000000000000000000",
      number: 1,
      //parentHash: "0x04540257811b46d103d9896e7807040e7de5080e285841c5430d1a81588a0ce4",
      receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
      // size: 535,
      stateRoot: "0x0000000000000000000000000000000000000000000000000000000000000000",
      //timestamp: 1595012243836,
      totalDifficulty: null,
      //transactions: [],
      transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
      //uncles: []
    });
    previousBlock = block;

    expect(block.transactions).to.be.a("array").empty;
    expect(block.uncles).to.be.a("array").empty;
    expect((block as any).sealFields).to.eql([
      "0x0000000000000000000000000000000000000000000000000000000000000000",
      "0x0000000000000000",
    ]);
    expect(block.hash).to.be.a("string").lengthOf(66);
    expect(block.parentHash).to.be.a("string").lengthOf(66);
    expect(block.timestamp).to.be.a("number");
  });

  it("get block by hash", async function () {
    const latest_block = await context.web3.eth.getBlock("latest");
    const block = await context.web3.eth.getBlock(latest_block.hash);
    expect(block.hash).to.be.eq(latest_block.hash);
  });

  it("get block by number", async function () {
    const block = await context.web3.eth.getBlock(1);
    expect(block).not.null;
  });

  it("should include previous block hash as parent (block 2)", async function () {
    this.timeout(15000);
    await createAndFinalizeBlock(context.web3);
    const block = await context.web3.eth.getBlock("latest");
    expect(block.hash).to.not.equal(previousBlock.hash);
    expect(block.parentHash).to.equal(previousBlock.hash);
  });

  it.skip("should include a tx in the block (block 3)", async function () {
    const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
    const GENESIS_ACCOUNT_BALANCE = "340282366920938463463374607431768211455";
    const GENESIS_ACCOUNT_PRIVATE_KEY =
      "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
    const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
    const NUMBER_OF_TRANSACTIONS: number = 1
    const basicTransfertx: TransactionConfig={
      from: GENESIS_ACCOUNT,
      to: TEST_ACCOUNT,
      value: "0x200", // Must me higher than ExistentialDeposit (500)
      gasPrice: "0x01",
      gas: "0x100000",
    }
    
    this.timeout(20000);

    const thousand=new Array(NUMBER_OF_TRANSACTIONS).fill(1)

    const txList:SignedTransaction[] = await Promise.all(thousand.map(async()=>{
      return await context.web3.eth.accounts.signTransaction(
        basicTransfertx,
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
    })) 
    const txRawList:string[]=txList.map((tx:SignedTransaction)=>{
      return tx.rawTransaction
    })
    await customRequest(context.web3, "eth_sendRawTransaction", txRawList);
    await createAndFinalizeBlock(context.web3);
    const block = await context.web3.eth.getBlock("latest");
    //console.log(block)
    expect(block.number).to.eq(3)
  });

  it("should include previous block hash as parent (block 3)", async function () {
    const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
    //const GENESIS_ACCOUNT_BALANCE = "340282366920938463463374607431768211455";
    const GENESIS_ACCOUNT_PRIVATE_KEY =
      "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
    const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
    //const NUMBER_OF_TRANSACTIONS: number = 1
    const basicTransfertx: TransactionConfig={
      from: GENESIS_ACCOUNT,
      to: TEST_ACCOUNT,
      value: "0x200", // Must me higher than ExistentialDeposit (500)
      gasPrice: "0x01",
      gas: "0x100000",
    }
    // const basicTransfertx2: TransactionConfig={
    //   from: GENESIS_ACCOUNT,
    //   to: TEST_ACCOUNT,
    //   value: "0x2000", // Must me higher than ExistentialDeposit (500)
    //   gasPrice: "0x01",
    //   gas: "0x100000",
    // }
    
    this.timeout(20000);

    async function fillBlockWithTx(numberOfTx:number, expectFunction, startingNonce:number){
      console.log('filling block with ',numberOfTx,' transactions')

      let nonce:number=startingNonce

      const numberArray=new Array(numberOfTx).fill(1)
        
      await Promise.all(numberArray.map(async(_,i)=>{
        console.log('another one',nonce+i)
        const tx:SignedTransaction = await context.web3.eth.accounts.signTransaction(
          {...basicTransfertx,nonce:nonce+i},
          GENESIS_ACCOUNT_PRIVATE_KEY
        );
        return customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      })) 

      await createAndFinalizeBlock(context.web3);
      const block = await context.web3.eth.getBlock("latest");
      console.log(block)
      expectFunction(block.transactions.length).to.eq(numberOfTx)
    }
    await fillBlockWithTx(5,expect,0)
    await fillBlockWithTx(50,expect,0)
    await fillBlockWithTx(500,expect,0)
    await fillBlockWithTx(5000,expect,0)
  });
});
