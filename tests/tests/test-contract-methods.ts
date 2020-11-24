import { expect } from "chai";

import {
  expectRevert, // Assertions for transactions that should fail
} from '@openzeppelin/test-helpers'

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { FIRST_CONTRACT_ADDRESS, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, INFINITE_CONTRACT_ABI, INFINITE_CONTRACT_ABI_VAR, INFINITE_CONTRACT_BYTECODE, INFINITE_CONTRACT_BYTECODE_VAR, TEST_CONTRACT_ABI, TEST_CONTRACT_BYTECODE } from "./constants";

describeWithMoonbeam("Moonbeam RPC (Contract Methods)", `simple-specs.json`, (context) => {

  before("create the contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.web3);
  });

  it("get transaction by hash", async () => {
    const latestBlock = await context.web3.eth.getBlock("latest");
    expect(latestBlock.transactions.length).to.equal(1);

    const tx_hash = latestBlock.transactions[0];
    const tx = await context.web3.eth.getTransaction(tx_hash);
    expect(tx.hash).to.equal(tx_hash);
  });

  it("should return contract method result", async function () {
    const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
      gasPrice: "0x01",
    });

    expect(await contract.methods.multiply(3).call()).to.equal("21");
  });


  it("inifinite loop", async function () {
    this.timeout(0);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: INFINITE_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    //console.log('tx',tx)
    let res=await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    //console.log('customreq',res);
    await createAndFinalizeBlock(context.web3); 
    let res2=await context.web3.eth.getTransactionReceipt((tx).transactionHash)
    //console.log('res2',res2)
    const contract = new context.web3.eth.Contract([INFINITE_CONTRACT_ABI],res2.contractAddress)
    //should revert with out of gas error
    //console.log(await contract.methods.infinite().call({from:GENESIS_ACCOUNT}))

    //expectRevert(contract.methods.infinite().call(),'evm error: OutOfGas');
    //expect(await contract.methods.infinite().call()).to.throw('evm error: OutOfGas');

      let bytesCode:string=await contract.methods.infinite().encodeABI()
      console.log('bytes',bytesCode)
      const contractCall = {
        from: GENESIS_ACCOUNT,
        data: bytesCode,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      };
      const txCall = await context.web3.eth.accounts.signTransaction(
        contractCall,
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      try{
        let res=await customRequest(context.web3, "eth_sendRawTransaction", [txCall.rawTransaction])
        console.log('res',res)
        await createAndFinalizeBlock(context.web3);
        let block = await context.web3.eth.getBlock("latest");
        console.log(block)
      } catch(e){
        console.log('errorattrappee',e)
      }
      console.log('FINI')
      

    await contract.methods
      .infinite()
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm error: OutOfGas`
        )
      );
      // try{
      //   await contract.methods.infinite().call()
      // } catch(e){
      //   //console.log('error caught : ',e)
      //   expect(e.toString()).to.eq(`Error: Returned error: evm revert: Reverted`)
      // }
  });

  it.skip("inifinite loop with incr", async function () {
    this.timeout(10000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: INFINITE_CONTRACT_BYTECODE_VAR,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    //console.log('tx',tx)
    let res=await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    //console.log('customreq',res);
    await createAndFinalizeBlock(context.web3); 
    let res2=await context.web3.eth.getTransactionReceipt((tx).transactionHash)
    //console.log('res2',res2)
    const contract = new context.web3.eth.Contract(INFINITE_CONTRACT_ABI_VAR,res2.contractAddress)
    //should revert with out of gas error
    //console.log(await contract.methods.infinite().call({from:GENESIS_ACCOUNT}))

    expectRevert(contract.methods.infinite().call(),'evm error: OutOfGas');
  });

  // Requires error handling
  it("should fail for missing parameters", async function () {
    const contract = new context.web3.eth.Contract(
      [{ ...TEST_CONTRACT_ABI, inputs: [] }],
      FIRST_CONTRACT_ADDRESS,
      {
        from: GENESIS_ACCOUNT,
        gasPrice: "0x01",
      }
    );
    await contract.methods
      .multiply()
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm revert: Reverted`
        )
      );
      //expect(async ()=>{return await contract.methods.multiply().call()}).to.throw(`Returned error: evm revert: Reverted`);
      try{
        await contract.methods.multiply().call()
      } catch(e){
        //console.log('error caught : ',e)
        expect(e.toString()).to.eq(`Error: Returned error: evm revert: Reverted`)
      }
      // expectRevert( contract.methods
      //   .multiply()
      //   .call(),`Returned error: evm revert: Reverted`)
  });

  // Requires error handling
  it("should fail for too many parameters", async function () {
    const contract = new context.web3.eth.Contract(
      [
        {
          ...TEST_CONTRACT_ABI,
          inputs: [
            { internalType: "uint256", name: "a", type: "uint256" },
            { internalType: "uint256", name: "b", type: "uint256" },
          ],
        },
      ],
      FIRST_CONTRACT_ADDRESS,
      {
        from: GENESIS_ACCOUNT,
        gasPrice: "0x01",
      }
    );
    await contract.methods
      .multiply(3, 4)
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm revert: Reverted`
        )
      );
  });

  // Requires error handling
  it("should fail for invalid parameters", async function () {
    const contract = new context.web3.eth.Contract(
      [
        {
          ...TEST_CONTRACT_ABI,
          inputs: [
            {
              internalType: "address",
              name: "a",
              type: "address",
            },
          ],
        },
      ],
      FIRST_CONTRACT_ADDRESS,
      { from: GENESIS_ACCOUNT, gasPrice: "0x01" }
    );
    await contract.methods
      .multiply("0x0123456789012345678901234567890123456789")
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm revert: Reverted`
        )
      );
  });
});
