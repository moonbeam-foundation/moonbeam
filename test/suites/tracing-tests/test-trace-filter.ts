import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ALITH_CONTRACT_ADDRESSES, alith } from "@moonwall/util";

describeSuite({
  id: "D3613",
  title: "Trace filter - Contract creation ",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    // Setup: Create 4 blocks with TraceFilter contracts
    beforeAll(async () => {
      const { contractAddress } = await context.deployContract!("TraceFilter", {
        args: [false],
      });
      await context.deployContract!("TraceFilter", {
        gas: 90_000n,
        args: [true],
      });

      await context.deployContract!("TraceFilter", {
        args: [false],
      });
      await context.deployContract!("TraceFilter", {
        args: [false],
      });

      const rawTx3 = await context.writeContract!({
        contractAddress,
        contractName: "TraceFilter",
        functionName: "subcalls",

        args: [ALITH_CONTRACT_ADDRESSES[2], ALITH_CONTRACT_ADDRESSES[3]],
        gas: 1000_000n,
        rawTxOnly: true,
      });
      await context.createBlock(rawTx3, { allowFailures: false });
    });

    it({
      id: "T01",
      title: "should be able to replay deployed contract",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x01",
            toBlock: "0x01",
          },
        ]);

        const transactionHash = (await context.viem().getBlock({ blockNumber: 1n }))
          .transactions[0];

        expect(response.length).to.equal(1);
        expect(response[0].action).to.include({
          creationMethod: "create",
          from: ALITH_ADDRESS.toLocaleLowerCase(),
          gas: "0x9a679",
          value: "0x0",
        });
        expect(response[0].result).to.include({
          address: ALITH_CONTRACT_ADDRESSES[0].toLocaleLowerCase(),
          gasUsed: "0x14fff", // TODO : Compare with value from another (comparable) network.
        });

        expect(response[0]).to.include({
          blockNumber: 1,
          subtraces: 0,
          transactionHash: transactionHash,
          transactionPosition: 0,
          type: "create",
        });
      },
    });

    it({
      id: "T02",
      title: "should be able to replay reverted contract",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x02",
            toBlock: "0x02",
          },
        ]);

        const transactionHash = (await context.web3().eth.getBlock(2)).transactions[0];

        expect(response.length).to.equal(1);
        expect(response[0].action.creationMethod).to.equal("create");
        expect(response[0].action.from).to.equal(ALITH_ADDRESS.toLocaleLowerCase());
        expect(response[0].action.gas).to.equal("0x10df");
        expect(response[0].action.init).to.be.a("string");
        expect(response[0].action.value).to.equal("0x0");
        expect(response[0].blockHash).to.be.a("string");
        expect(response[0].blockNumber).to.equal(2);
        expect(response[0].result).to.equal(undefined);
        expect(response[0].error).to.equal("Reverted");
        expect(response[0].subtraces).to.equal(0);
        expect(response[0].traceAddress.length).to.equal(0);
        expect(response[0].transactionHash).to.equal(transactionHash);
        expect(response[0].transactionPosition).to.equal(0);
        expect(response[0].type).to.equal("create");
      },
    });

    it({
      id: "T03",
      title: "should be able to trace through multiple blocks",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x02",
            toBlock: "0x04",
          },
        ]);

        expect(response.length).to.equal(3);
        expect(response[0].blockNumber).to.equal(2);
        expect(response[0].transactionPosition).to.equal(0);
        expect(response[1].blockNumber).to.equal(3);
        expect(response[1].transactionPosition).to.equal(0);
        expect(response[2].blockNumber).to.equal(4);
        expect(response[2].transactionPosition).to.equal(0);
      },
    });

    it({
      id: "T04",
      title: "should be able to trace sub-call with reverts",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x05",
            toBlock: "0x05",
          },
        ]);

        expect(response.length).to.equal(7);
        expect(response[0].subtraces).to.equal(2);
        expect(response[0].traceAddress).to.deep.equal([]);
        expect(response[1].subtraces).to.equal(2);
        expect(response[1].traceAddress).to.deep.equal([0]);
        expect(response[2].subtraces).to.equal(0);
        expect(response[2].traceAddress).to.deep.equal([0, 0]);
        expect(response[3].subtraces).to.equal(0);
        expect(response[3].traceAddress).to.deep.equal([0, 1]);
        expect(response[4].subtraces).to.equal(2);
        expect(response[4].traceAddress).to.deep.equal([1]);
        expect(response[5].subtraces).to.equal(0);
        expect(response[5].traceAddress).to.deep.equal([1, 0]);
        expect(response[6].subtraces).to.equal(0);
        expect(response[6].traceAddress).to.deep.equal([1, 1]);
      },
    });

    it({
      id: "T05",
      title: "should support tracing range of blocks",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x03",
            toBlock: "0x05",
          },
        ]);

        expect(response.length).to.equal(9);
        expect(response[0].blockNumber).to.equal(3);
        expect(response[0].transactionPosition).to.equal(0);
        expect(response[1].blockNumber).to.equal(4);
        expect(response[1].transactionPosition).to.equal(0);
        expect(response[2].blockNumber).to.equal(5);
        expect(response[2].transactionPosition).to.equal(0);
        expect(response[3].blockNumber).to.equal(5);
        expect(response[3].transactionPosition).to.equal(0);
        expect(response[4].blockNumber).to.equal(5);
        expect(response[4].transactionPosition).to.equal(0);
        expect(response[5].blockNumber).to.equal(5);
        expect(response[5].transactionPosition).to.equal(0);
        expect(response[6].blockNumber).to.equal(5);
        expect(response[6].transactionPosition).to.equal(0);
        expect(response[7].blockNumber).to.equal(5);
        expect(response[7].transactionPosition).to.equal(0);
        expect(response[8].blockNumber).to.equal(5);
        expect(response[8].transactionPosition).to.equal(0);
      },
    });

    it({
      id: "T06",
      title: "should support filtering trace per fromAddress",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x03",
            toBlock: "0x05",
            fromAddress: [alith.address],
          },
        ]);

        expect(response.length).to.equal(3);
      },
    });

    it({
      id: "T07",
      title: "should support filtering trace per toAddress",
      test: async function () {
        const response = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x03",
            toBlock: "0x05",
            toAddress: [ALITH_CONTRACT_ADDRESSES[3]],
          },
        ]);

        expect(response.length).to.equal(4);
      },
    });

    it({
      id: "T08",
      title: "should succeed for 500 traces request",
      test: async function () {
        await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x01",
            toBlock: "0x04",
            count: 500,
          },
        ]).catch(() => {
          expect.fail("should not fail");
        });
      },
    });

    it({
      id: "T09",
      title: "should fail for 501 traces request",
      test: async function () {
        await customDevRpcRequest("trace_filter", [
          {
            fromBlock: "0x01",
            toBlock: "0x04",
            count: 501,
          },
        ]).then(
          () => {
            expect.fail("should not succeed");
          },
          (error) => {
            expect(error.message).to.eq("count (501) can't be greater than maximum (500)");
          }
        );
      },
    });
  },
});
