import { expect } from "chai";

import { TransactionReceipt } from "web3-core";

import { callContractFunctionMS, deployContractManualSeal, describeWithMoonbeam } from "./util";
import {
  FINITE_LOOP_CONTRACT_ABI,
  FINITE_LOOP_CONTRACT_BYTECODE,
  INFINITE_CONTRACT_ABI,
  INFINITE_CONTRACT_ABI_VAR,
  INFINITE_CONTRACT_BYTECODE,
  INFINITE_CONTRACT_BYTECODE_VAR,
  TEST_CONTRACT_BYTECODE_INCR,
  TEST_CONTRACT_INCR_ABI,
} from "./constants";

describeWithMoonbeam("Moonbeam RPC (Contract Loops)", `simple-specs.json`, (context) => {
  it("should increment contract state - to check normal contract behavior", async function () {
    // // instantiate contract
    const contract = await deployContractManualSeal(
      context.polkadotApi,
      context.web3,
      TEST_CONTRACT_BYTECODE_INCR,
      TEST_CONTRACT_INCR_ABI
    );

    // check variable initializaion
    expect(await contract.methods.count().call()).to.eq("0");

    // call incr function
    let bytesCode: string = await contract.methods.incr().encodeABI();
    await callContractFunctionMS(context, contract.options.address, bytesCode);

    // check variable incrementation
    expect(await contract.methods.count().call()).to.eq("1");
  });

  it("inifinite loop call should return OutOfGas", async function () {
    //deploy infinite contract
    const contract = await deployContractManualSeal(
      context.polkadotApi,
      context.web3,
      INFINITE_CONTRACT_BYTECODE,
      [INFINITE_CONTRACT_ABI]
    );

    // call infinite loop
    await contract.methods
      .infinite()
      .call({ gas: "0x100000" })
      .catch((err) => expect(err.message).to.equal(`Returned error: out of gas or fund`));
  });

  it("inifinite loop send with incr should return OutOfGas", async function () {
    // deploy contract
    const contract = await deployContractManualSeal(
      context.polkadotApi,
      context.web3,
      INFINITE_CONTRACT_BYTECODE_VAR,
      INFINITE_CONTRACT_ABI_VAR
    );

    //make infinite loop function call
    let bytesCode: string = await contract.methods.infinite().encodeABI();
    try {
      await callContractFunctionMS(context, contract.options.address, bytesCode);
      let block = await context.web3.eth.getBlock("latest");
      const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(
        block.transactions[0]
      );
      expect(receipt.status).to.eq(false);
    } catch (e) {
      console.log("error caught", e);
      throw new Error(e);
    }
  });

  it("finite loop with incr: check gas usage, with normal gas limit,\
  should error before 700 loops", async function () {
    // For a normal 1048576 gas limit, loop should revert out of gas between 600 and 700 loops

    //deploy finite loop contract
    const contract = await deployContractManualSeal(
      context.polkadotApi,
      context.web3,
      FINITE_LOOP_CONTRACT_BYTECODE,
      FINITE_LOOP_CONTRACT_ABI
    );

    //make finite loop function call
    async function callLoopIncrContract(nb: number): Promise<number> {
      const startIncr: number = Number(await contract.methods.count().call());
      const bytesCode: string = await contract.methods.incr(nb).encodeABI();
      try {
        await callContractFunctionMS(context, contract.options.address, bytesCode);
        return Number(await contract.methods.count().call()) - startIncr;
      } catch (e) {
        console.log("error caught", e);
      }
    }
    // 1 loop to make sure it works
    expect(await callLoopIncrContract(1)).to.eq(1);
    let block = await context.web3.eth.getBlock("latest");
    expect(block.gasUsed).to.eq(42343); //check that gas costs stay the same

    // // 600 loop
    expect(await callLoopIncrContract(600)).to.eq(600);
    block = await context.web3.eth.getBlock("latest");
    expect(block.gasUsed).to.eq(1024084); //check that gas costs stay the same

    // 700 loop should revert out of gas
    expect(await callLoopIncrContract(700)).to.eq(0);
    block = await context.web3.eth.getBlock("latest");
    expect(block.gasUsed).to.eq(1048576); //check that gas is the gas limit
    const receipt: TransactionReceipt = await context.web3.eth.getTransactionReceipt(
      block.transactions[0]
    );
    expect(receipt.status).to.eq(false);
  });
  // TODO : add test when we have a block limit
});
