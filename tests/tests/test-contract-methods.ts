import { expect } from "chai";

import { TransactionReceipt } from "web3-core";

import {
  createAndFinalizeBlock,
  customRequest,
  deployContractManualSeal,
  describeWithMoonbeam,
} from "./util";
import {
  FINITE_LOOP_CONTRACT_ABI,
  FINITE_LOOP_CONTRACT_BYTECODE,
  FIRST_CONTRACT_ADDRESS,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  INFINITE_CONTRACT_ABI,
  INFINITE_CONTRACT_ABI_VAR,
  INFINITE_CONTRACT_BYTECODE,
  INFINITE_CONTRACT_BYTECODE_VAR,
  TEST_CONTRACT_ABI,
  TEST_CONTRACT_BYTECODE,
  TEST_CONTRACT_BYTECODE_INCR,
  TEST_CONTRACT_INCR_ABI,
} from "./constants";

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

  it.only("should increment contract state", async function () {
    // // instantiate contract
    const contract = await deployContractManualSeal(
      context.web3,
      TEST_CONTRACT_BYTECODE_INCR,
      TEST_CONTRACT_INCR_ABI,
      GENESIS_ACCOUNT,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    // check variable initializaion
    expect(await contract.methods.count().call()).to.eq("0");

    // call incr function
    let bytesCode: string = await contract.methods.incr().encodeABI();
    const contractCall = {
      from: GENESIS_ACCOUNT,
      to: contract.options.address,
      data: bytesCode,
      gasPrice: "0x01",
      gas: "0x100000",
    };
    const txCall = await context.web3.eth.accounts.signTransaction(
      contractCall,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [txCall.rawTransaction]);
    await createAndFinalizeBlock(context.web3);

    // check variable incrementation
    expect(await contract.methods.count().call()).to.eq("1");
  });

  it.skip("inifinite loop", async function () {
    this.timeout(0);

    // get nonce
    let nonce: number = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);

    //deploy infinite contract
    const contract = await deployContractManualSeal(
      context.web3,
      INFINITE_CONTRACT_BYTECODE,
      [INFINITE_CONTRACT_ABI],
      GENESIS_ACCOUNT,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    //make infinite loop function call
    let bytesCode: string = await contract.methods.infinite().encodeABI();
    const contractCall = {
      from: GENESIS_ACCOUNT,
      data: bytesCode,
      to: contract.options.address,
      gasPrice: "0x01",
      gas: "0x100000",
      nonce: nonce + 1,
    };
    const txCall = await context.web3.eth.accounts.signTransaction(
      contractCall,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    try {
      let res = await customRequest(context.web3, "eth_sendRawTransaction", [
        txCall.rawTransaction,
      ]);
      console.log("res", res);
      await createAndFinalizeBlock(context.web3);
      let block = await context.web3.eth.getBlock("latest");
      console.log(block);
    } catch (e) {
      console.log("error caught", e);
    }
    // TODO: this should throw an error

    // await contract.methods
    //   .infinite()
    //   .call()
    //   .catch((err) =>
    //     expect(err.message).to.equal(
    //       `Returned error: evm error: OutOfGas`
    //     )
    //   );
    // try{
    //   await contract.methods.infinite().call()
    // } catch(e){
    //   //console.log('error caught : ',e)
    //   expect(e.toString()).to.eq(`Error: Returned error: evm revert: Reverted`)
    // }
  });

  it.skip("inifinite loop with incr", async function () {
    this.timeout(0);

    // deploy contract
    const contract = await deployContractManualSeal(
      context.web3,
      INFINITE_CONTRACT_BYTECODE_VAR,
      INFINITE_CONTRACT_ABI_VAR,
      GENESIS_ACCOUNT,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    //make infinite loop function call
    let bytesCode: string = await contract.methods.infinite().encodeABI();
    const contractCall = {
      from: GENESIS_ACCOUNT,
      data: bytesCode,
      to: contract.options.address,
      gasPrice: "0x01",
      gas: "0x100000",
    };
    const txCall = await context.web3.eth.accounts.signTransaction(
      contractCall,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    try {
      let res = await customRequest(context.web3, "eth_sendRawTransaction", [
        txCall.rawTransaction,
      ]);
      console.log("res", res);
      await createAndFinalizeBlock(context.web3);
      let block = await context.web3.eth.getBlock("latest");
      console.log(block);
      console.log("data", await contract.methods.data().call());
    } catch (e) {
      console.log("error caught", e);
    }
    // TODO: this should throw an error
  });

  it.only("finite loop with incr: check gas usage", async function () {
    this.timeout(0);

    //deploy finite loop contract
    const contract = await deployContractManualSeal(
      context.web3,
      FINITE_LOOP_CONTRACT_BYTECODE,
      FINITE_LOOP_CONTRACT_ABI,
      GENESIS_ACCOUNT,
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    //make finite loop function call
    async function callLoopIncrContract(nb: number): Promise<number> {
      const startIncr: number = Number(await contract.methods.count().call());
      const bytesCode: string = await contract.methods.incr(nb).encodeABI();
      const contractCall = {
        from: GENESIS_ACCOUNT,
        data: bytesCode,
        to: contract.options.address,
        gasPrice: "0x01",
        gas: "0x100000", //gas: "0x100000", //TODO: exceeding the gas limit should throw some kind of error
      };
      const txCall = await context.web3.eth.accounts.signTransaction(
        contractCall,
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      try {
        let res = await customRequest(context.web3, "eth_sendRawTransaction", [
          txCall.rawTransaction,
        ]);
        await createAndFinalizeBlock(context.web3);
        return Number(await contract.methods.count().call()) - startIncr;
      } catch (e) {
        console.log("error caught", e);
      }
    }
    // 1 loop
    expect(await callLoopIncrContract(1)).to.eq(1);
    let block = await context.web3.eth.getBlock("latest");
    console.log("1 block gas used", block.gasUsed);

    // // 10 loop
    // expect(await callLoopIncrContract(10)).to.eq(10)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('10 block gas used',block.gasUsed);

    // // 20 loop
    // expect(await callLoopIncrContract(20)).to.eq(20)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('10 block gas used',block.gasUsed);

    // // 100 loop
    // expect(await callLoopIncrContract(100)).to.eq(100)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('100 block gas used',block.gasUsed);

    // // 200 loop
    // expect(await callLoopIncrContract(200)).to.eq(200)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('200 block gas used',block.gasUsed);

    // // 500 loop
    // expect(await callLoopIncrContract(500)).to.eq(500)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('500 block gas used',block.gasUsed);

    // // 600 loop
    // expect(await callLoopIncrContract(600)).to.eq(600)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('600 block gas used',block.gasUsed);
    console.log(
      "OOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOO+++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++++"
    );
    // 700 loop
    expect(await callLoopIncrContract(700)).to.eq(700);
    block = await context.web3.eth.getBlock("latest");
    console.log("700 block gas used", block.gasUsed);

    // //1000 loop
    // expect(await callLoopIncrContract(1000)).to.eq(1000)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('block gas used',block.gasUsed);

    // // 10 000 loop
    // expect(await callLoopIncrContract(10000)).to.eq(10000)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('block gas used',block.gasUsed);

    // // 1000 000 loop
    // expect(await callLoopIncrContract(1000000)).to.eq(1000000)
    // block = await context.web3.eth.getBlock("latest");
    // console.log('1000 000 block gas used',block.gasUsed);
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
      .catch((err) => expect(err.message).to.equal(`Returned error: evm revert: Reverted`));
    //expect(async ()=>{return await contract.methods.multiply().call()}).to.throw(`Returned error: evm revert: Reverted`);
    try {
      await contract.methods.multiply().call();
    } catch (e) {
      expect(e.toString()).to.eq(`Error: Returned error: evm revert: Reverted`);
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
      .catch((err) => expect(err.message).to.equal(`Returned error: evm revert: Reverted`));
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
      .catch((err) => expect(err.message).to.equal(`Returned error: evm revert: Reverted`));
  });
});
