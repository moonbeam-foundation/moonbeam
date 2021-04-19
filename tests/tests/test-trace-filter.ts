import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";
import { GENESIS_ACCOUNT } from "../util/constants";

const GENESIS_CONTRACT_ADDRESSES = [
  "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a",
  "0x42e2ee7ba8975c473157634ac2af4098190fc741",
  "0xf8cef78e923919054037a1d03662bbd884ff4edf",
];

describeDevMoonbeam("Trace filter - Contract creation ", (context) => {
  before("Setup: Create 4 blocks with TraceFilter contracts", async function () {
    const { contract, rawTx } = await createContract(context.web3, "TraceFilter", {}, [false]);
    await context.createBlock({ transactions: [rawTx] });

    const { rawTx: rawTx2 } = await createContract(context.web3, "TraceFilter", {}, [true]);
    await context.createBlock({ transactions: [rawTx2] });

    const { rawTx: rawTx3 } = await createContract(context.web3, "TraceFilter", {}, [false]);
    const { rawTx: rawTx4 } = await createContract(context.web3, "TraceFilter", { nonce: 3 }, [
      false,
    ]);
    await context.createBlock({ transactions: [rawTx3, rawTx4] });

    await context.createBlock({
      transactions: [
        await createContractExecution(context.web3, {
          contract,
          contractCall: contract.methods.subcalls(
            GENESIS_CONTRACT_ADDRESSES[1],
            GENESIS_CONTRACT_ADDRESSES[2]
          ),
        }),
      ],
    });
  });

  it("should be able to replay deployed contract", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x01",
        toBlock: "0x01",
      },
    ]);

    expect(response.result.length).to.equal(1);
    expect(response.result[0].action).to.include({
      createMethod: "create",
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gas: "0xb718d7",
      value: "0x0",
    });
    expect(response.result[0].result).to.include({
      address: "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a",
      gasUsed: "0x229",
    });

    expect(response.result[0]).to.include({
      blockNumber: 1,
      subtraces: 0,
      transactionHash: "0x5301ed3a9a1be6001cf261f4197169fd6bc24804270be3c7de19fffdb63ad198",
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

    expect(response.result.length).to.equal(1);
    expect(response.result[0].action.createMethod).to.equal("create");
    expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
    expect(response.result[0].action.gas).to.equal("0xb7198c");
    expect(response.result[0].action.input).to.be.a("string");
    expect(response.result[0].action.value).to.equal("0x0");
    expect(response.result[0].blockHash).to.be.a("string");
    expect(response.result[0].blockNumber).to.equal(2);
    expect(response.result[0].result).to.equal(undefined);
    expect(response.result[0].error).to.equal("Reverted");
    expect(response.result[0].subtraces).to.equal(0);
    expect(response.result[0].traceAddress.length).to.equal(0);
    expect(response.result[0].transactionHash).to.equal(
      "0x0ddcb527475b0d5e6a45ba6d9bb367c18a7142b5919247f5dd521c744fcd22a3"
    );
    expect(response.result[0].transactionPosition).to.equal(0);
    expect(response.result[0].type).to.equal("create");
  });

  it("Multiple transactions in the same block + trace over multiple blocks", async function () {
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

  it("Call with subcalls, some reverting", async function () {
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

  it("Request range of blocks", async function () {
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

  it("Filter fromAddress", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        fromAddress: [GENESIS_ACCOUNT],
      },
    ]);

    expect(response.result.length).to.equal(3);
  });

  it("Filter toAddress", async function () {
    let response = await customWeb3Request(context.web3, "trace_filter", [
      {
        fromBlock: "0x03",
        toBlock: "0x04",
        toAddress: [GENESIS_CONTRACT_ADDRESSES[2]],
      },
    ]);

    expect(response.result.length).to.equal(4);
  });
});
