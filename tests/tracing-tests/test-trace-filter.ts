import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";
import { GENESIS_ACCOUNT } from "../util/constants";

const GENESIS_CONTRACT_ADDRESSES = [
  "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a",
  "0x42e2ee7ba8975c473157634ac2af4098190fc741",
  "0xf8cef78e923919054037a1d03662bbd884ff4edf",
];

describeDevMoonbeamAllEthTxTypes("Trace filter - Contract creation ", (context) => {
  before("Setup: Create 4 blocks with TraceFilter contracts", async function () {
    const { contract, rawTx } = await createContract(context, "TraceFilter", {}, [false]);
    await context.createBlock({ transactions: [rawTx] });

    const { rawTx: rawTx2 } = await createContract(context, "TraceFilter", {}, [true]);
    await context.createBlock({ transactions: [rawTx2] });

    const { rawTx: rawTx3 } = await createContract(context, "TraceFilter", {}, [false]);
    const { rawTx: rawTx4 } = await createContract(context, "TraceFilter", { nonce: 3 }, [
      false,
    ]);
    await context.createBlock({ transactions: [rawTx3, rawTx4] });

    await context.createBlock({
      transactions: [
        await createContractExecution(context, {
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
      creationMethod: "create",
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gas: "0xb60b27",
      value: "0x0",
    });
    expect(response.result[0].result).to.include({
      address: "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a",
      gasUsed: "0x10fd9", // TODO : Compare with value from another (comparable) network.
    });

    expect(response.result[0]).to.include({
      blockNumber: 1,
      subtraces: 0,
      transactionHash: "0x38543a19a4fdf101ff6607f712a2283e0056d849f7dbe36715b464c6b08e317e",
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
    expect(response.result[0].action.creationMethod).to.equal("create");
    expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
    expect(response.result[0].action.gas).to.equal("0xb60bd0");
    expect(response.result[0].action.init).to.be.a("string");
    expect(response.result[0].action.value).to.equal("0x0");
    expect(response.result[0].blockHash).to.be.a("string");
    expect(response.result[0].blockNumber).to.equal(2);
    expect(response.result[0].result).to.equal(undefined);
    expect(response.result[0].error).to.equal("Reverted");
    expect(response.result[0].subtraces).to.equal(0);
    expect(response.result[0].traceAddress.length).to.equal(0);
    expect(response.result[0].transactionHash).to.equal(
      "0xe910be3a7b2de6bde555be5ac30d79189b1e000cb09bf0591b05972f6d9052eb"
    );
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
        fromAddress: [GENESIS_ACCOUNT],
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
