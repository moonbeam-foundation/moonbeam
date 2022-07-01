import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, ALITH_ADDRESS, ALITH_CONTRACT_ADDRESSES } from "../util/accounts";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";

const GENESIS_CONTRACT_ADDRESSES = [
  ALITH_CONTRACT_ADDRESSES[0],
  ALITH_CONTRACT_ADDRESSES[2],
  ALITH_CONTRACT_ADDRESSES[3],
];

describeDevMoonbeam("Trace filter - Contract creation ", (context) => {
  before("Setup: Create 4 blocks with TraceFilter contracts", async function () {
    const { contract, rawTx } = await createContract(context, "TraceFilter", {}, [false]);
    await context.createBlock(rawTx);

    const { rawTx: rawTx2 } = await createContract(context, "TraceFilter", { gas: 90_000 }, [true]);

    await context.createBlock([rawTx2]);

    const { rawTx: rawTx3 } = await createContract(context, "TraceFilter", {}, [false]);
    const { rawTx: rawTx4 } = await createContract(context, "TraceFilter", { nonce: 3 }, [false]);
    await context.createBlock([rawTx3, rawTx4]);

    await context.createBlock(
      createContractExecution(context, {
        contract,
        contractCall: contract.methods.subcalls(
          GENESIS_CONTRACT_ADDRESSES[1],
          GENESIS_CONTRACT_ADDRESSES[2]
        ),
      })
    );
  });

  it("should be able to replay deployed contract", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x01",
      },
    ]);

    const transactionHash = (await context.web3.eth.getBlock(1)).transactions[0];

    expect(response.result.length).to.equal(1);
    expect(response.result[0].action).to.include({
      creationMethod: "create",
      from: ALITH_ADDRESS,
      gas: "0x6bdea",
      value: "0x0",
    });
    expect(response.result[0].result).to.include({
      address: ALITH_CONTRACT_ADDRESSES[0].toLocaleLowerCase(),
      gasUsed: "0x159c6", // TODO : Compare with value from another (comparable) network.
    });

    expect(response.result[0]).to.include({
      blockNumber: 1,
      subtraces: 0,
      transactionHash: transactionHash,
      transactionPosition: 0,
      type: "create",
    });
  });

  it("should be able to replay reverted contract", async function () {
    // Perform RPC call.
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x02",
        toBlock: "0x02",
      },
    ]);

    const transactionHash = (await context.web3.eth.getBlock(2)).transactions[0];

    expect(response.result.length).to.equal(1);
    expect(response.result[0].action.creationMethod).to.equal("create");
    expect(response.result[0].action.from).to.equal(ALITH_ADDRESS);
    expect(response.result[0].action.gas).to.equal("0x758");
    expect(response.result[0].action.init).to.be.a("string");
    expect(response.result[0].action.value).to.equal("0x0");
    expect(response.result[0].blockHash).to.be.a("string");
    expect(response.result[0].blockNumber).to.equal(2);
    expect(response.result[0].result).to.equal(undefined);
    expect(response.result[0].error).to.equal("Reverted");
    expect(response.result[0].subtraces).to.equal(0);
    expect(response.result[0].traceAddress.length).to.equal(0);
    expect(response.result[0].transactionHash).to.equal(transactionHash);
    expect(response.result[0].transactionPosition).to.equal(0);
    expect(response.result[0].type).to.equal("create");
  });

  it("should be able to trace through multiple blocks", async function () {
    // Perform RPC call.
    let response = await customWeb3Request(context.web3, "trace_filter", [
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
  });

  it("should be able to trace sub-call with reverts", async function () {
    // Perform RPC call.
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x04",
        toBlock: "0x04",
      },
    ]);

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

  it("should support tracing range of blocks", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
      },
    ]);

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

  it("should support filtering trace per fromAddress", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        fromAddress: [alith.address],
      },
    ]);

    expect(response.result.length).to.equal(3);
  });

  it("should support filtering trace per toAddress", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        toAddress: [GENESIS_CONTRACT_ADDRESSES[2]],
      },
    ]);

    expect(response.result.length).to.equal(4);
  });

  it("should handle pagination", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        count: 2,
        after: 1,
      },
    ]);

    expect(response.result.length).to.equal(2);
    expect(response.result[0].blockNumber).to.equal(3);
    expect(response.result[0].transactionPosition).to.equal(1);
    expect(response.result[1].blockNumber).to.equal(4);
    expect(response.result[1].transactionPosition).to.equal(0);
  });

  it("should succeed for 500 traces request", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x04",
        count: 500,
      },
    ]);
    expect(response.error).to.not.exist;
  });

  it("should fail for 501 traces request", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x04",
        count: 501,
      },
    ]);
    expect(response.error).to.deep.eq({
      code: -32603,
      message: "count (501) can't be greater than maximum (500)",
    });
  });
});
