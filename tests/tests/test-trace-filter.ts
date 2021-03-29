import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

const CONTRACT = require("./constants/TraceFilter.json");

const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GENESIS_ACCOUNT_PRIVATE_KEY =
  "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

const address0 = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";
const address1 = "0x42e2ee7ba8975c473157634ac2af4098190fc741";
const address2 = "0xf8cef78e923919054037a1d03662bbd884ff4edf";

describeWithMoonbeam("Moonbeam RPC (trace_filter)", `simple-specs.json`, (context) => {
  step("Replay succeeding CREATE", async function () {
    // Deploy contract
    const contract = new context.web3.eth.Contract(CONTRACT.abi);
    const contract_deploy = contract.deploy({
      data: CONTRACT.bytecode,
      arguments: [false], // don't revert
    });

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: contract_deploy.encodeABI(),
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x500000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

    await createAndFinalizeBlock(context.polkadotApi);

    // Perform RPC call.
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x01",
      },
    ]);

    // console.log(JSON.stringify(response));

    expect(response.result.length).to.equal(1);
    expect(response.result[0].action.createMethod).to.equal("create");
    expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
    expect(response.result[0].action.gas).to.equal("0x4ffead");
    expect(response.result[0].action.input).to.be.a("string");
    expect(response.result[0].action.value).to.equal("0x0");
    expect(response.result[0].blockHash).to.be.a("string");
    expect(response.result[0].blockNumber).to.equal(1);
    expect(response.result[0].result.address).to.equal(
      "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a"
    );
    expect(response.result[0].result.code).to.be.a("string");
    expect(response.result[0].result.gasUsed).to.equal("0x153");
    expect(response.result[0].error).to.equal(undefined);
    expect(response.result[0].subtraces).to.equal(0);
    expect(response.result[0].traceAddress.length).to.equal(0);
    expect(response.result[0].transactionHash).to.equal(
      "0x282fdd0b08fd385bbc233cffd5679ee703fc6b4c5b54d6096ae47fa92372790e"
    );
    expect(response.result[0].transactionPosition).to.equal(0);
    expect(response.result[0].type).to.equal("create");
  });

  step("Replay reverting CREATE", async function () {
    // Deploy contract
    const contract = new context.web3.eth.Contract(CONTRACT.abi);
    const contract_deploy = contract.deploy({
      data: CONTRACT.bytecode,
      arguments: [true], // revert
    });

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: contract_deploy.encodeABI(),
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x500000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

    await createAndFinalizeBlock(context.polkadotApi);

    // Perform RPC call.
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x02",
        toBlock: "0x02",
      },
    ]);

    // console.log(JSON.stringify(response));

    expect(response.result.length).to.equal(1);
    expect(response.result[0].action.createMethod).to.equal("create");
    expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
    expect(response.result[0].action.gas).to.equal("0x4fff44");
    expect(response.result[0].action.input).to.be.a("string");
    expect(response.result[0].action.value).to.equal("0x0");
    expect(response.result[0].blockHash).to.be.a("string");
    expect(response.result[0].blockNumber).to.equal(2);
    expect(response.result[0].result).to.equal(undefined);
    expect(response.result[0].error).to.equal("Reverted");
    expect(response.result[0].subtraces).to.equal(0);
    expect(response.result[0].traceAddress.length).to.equal(0);
    expect(response.result[0].transactionHash).to.equal(
      "0x214cf6578d15751c7d5e68ad7167f2b7bcbb0023be155cd55cd1fb059e238c89"
    );
    expect(response.result[0].transactionPosition).to.equal(0);
    expect(response.result[0].type).to.equal("create");
  });

  step("Multiple transactions in the same block + trace over multiple blocks", async function () {
    const contract = new context.web3.eth.Contract(CONTRACT.abi);

    // Deploy 2 more contracts
    for (var i = 0; i < 2; i++) {
      const contract_deploy = contract.deploy({
        data: CONTRACT.bytecode,
        arguments: [false], // don't revert
      });

      const tx = await context.web3.eth.accounts.signTransaction(
        {
          nonce: 2 + i,
          from: GENESIS_ACCOUNT,
          data: contract_deploy.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    }

    await createAndFinalizeBlock(context.polkadotApi);

    // Perform RPC call.
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x02",
        toBlock: "0x03",
      },
    ]);

    expect(response.result.length).to.equal(3);
    expect(response.result[0].blockNumber).to.equal(2);
    expect(response.result[0].transactionPosition).to.equal(0);
    expect(response.result[1].blockNumber).to.equal(3);
    expect(response.result[1].transactionPosition).to.equal(0);
    expect(response.result[2].blockNumber).to.equal(3);
    expect(response.result[2].transactionPosition).to.equal(1);

    // console.log(JSON.stringify(response));
  });

  step("Call with subcalls, some reverting", async function () {
    const contract = new context.web3.eth.Contract(CONTRACT.abi);

    const contract_call = contract.methods.subcalls(address1, address2);

    const tx = await context.web3.eth.accounts.signTransaction(
      {
        to: address0,
        from: GENESIS_ACCOUNT,
        data: contract_call.encodeABI(),
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x500000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

    await createAndFinalizeBlock(context.polkadotApi);

    // Perform RPC call.
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x04",
        toBlock: "0x04",
      },
    ]);

    // console.log(JSON.stringify(response));
    expect(response.result.length).to.equal(7);
    expect(response.result[0].subtraces).to.equal(2);
    expect(response.result[0].traceAddress).to.deep.equal([]);
    expect(response.result[1].subtraces).to.equal(2);
    expect(response.result[1].traceAddress).to.deep.equal([0]);
    expect(response.result[2].subtraces).to.equal(0);
    expect(response.result[2].traceAddress).to.deep.equal([0, 0]);
    expect(response.result[3].subtraces).to.equal(0);
    expect(response.result[3].traceAddress).to.deep.equal([0, 1]);
    expect(response.result[4].subtraces).to.equal(2);
    expect(response.result[4].traceAddress).to.deep.equal([1]);
    expect(response.result[5].subtraces).to.equal(0);
    expect(response.result[5].traceAddress).to.deep.equal([1, 0]);
    expect(response.result[6].subtraces).to.equal(0);
    expect(response.result[6].traceAddress).to.deep.equal([1, 1]);
  });

  step("Request range of blocks", async function () {
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
      },
    ]);

    // console.log(JSON.stringify(response));
    expect(response.result.length).to.equal(9);
    expect(response.result[0].blockNumber).to.equal(3);
    expect(response.result[0].transactionPosition).to.equal(0);
    expect(response.result[1].blockNumber).to.equal(3);
    expect(response.result[1].transactionPosition).to.equal(1);
    expect(response.result[2].blockNumber).to.equal(4);
    expect(response.result[2].transactionPosition).to.equal(0);
    expect(response.result[3].blockNumber).to.equal(4);
    expect(response.result[3].transactionPosition).to.equal(0);
    expect(response.result[4].blockNumber).to.equal(4);
    expect(response.result[4].transactionPosition).to.equal(0);
    expect(response.result[5].blockNumber).to.equal(4);
    expect(response.result[5].transactionPosition).to.equal(0);
    expect(response.result[6].blockNumber).to.equal(4);
    expect(response.result[6].transactionPosition).to.equal(0);
    expect(response.result[7].blockNumber).to.equal(4);
    expect(response.result[7].transactionPosition).to.equal(0);
    expect(response.result[8].blockNumber).to.equal(4);
    expect(response.result[8].transactionPosition).to.equal(0);
  });

  step("Filter fromAddress", async function () {
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        fromAddress: [GENESIS_ACCOUNT],
      },
    ]);

    expect(response.result.length).to.equal(3);
  });

  step("Filter toAddress", async function () {
    let response = await customRequest(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        toAddress: [address2],
      },
    ]);

    expect(response.result.length).to.equal(4);
  });
});
