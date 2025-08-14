import "@moonbeam-network/api-augment";
import { numberToHex } from "@polkadot/util";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { type NetworkTestArtifact, tracingTxns } from "../../helpers/tracing-txns.js";
import type { ApiPromise } from "@polkadot/api";
import { rateLimiter } from "../../helpers/common.js";
import type { ethers } from "ethers";

const limiter = rateLimiter();

describeSuite({
  id: "S14",
  title: "Verifying historic compatibility",
  foundationMethods: "read_only",
  testCases: async ({ context, it, log }) => {
    let traceStatic: NetworkTestArtifact;
    let skipTest = { skip: false, networkName: "", chainId: "" };
    let blockNumber: number;
    let blockHash: string;
    let collatorAddress: string;
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const chainId = (await paraApi.query.ethereumChainId.chainId()).toString();
      log(`Loading test data for chainId ${chainId}.`);
      traceStatic = tracingTxns.find((a) => a.chainId.toString() === chainId)!;
      const networkName = (await paraApi.rpc.system.chain()).toString();
      const latestBlockNumberToCheck = traceStatic
        ? Math.max(...traceStatic.testData.map((d) => d.blockNumber))
        : 0;
      blockNumber = (await paraApi.query.ethereum.currentBlock()).unwrap().header.number.toNumber();
      if (
        !traceStatic ||
        networkName !== traceStatic.networkLabel ||
        latestBlockNumberToCheck > blockNumber
      ) {
        skipTest = { skip: true, networkName, chainId };
      }

      blockHash = (await paraApi.query.ethereum.blockHash(blockNumber)).toString();
      collatorAddress = (await paraApi.query.parachainStaking.selectedCandidates())[0].toString();
    });

    it({
      id: "C100",
      title: "can call debug_traceTransaction",
      timeout: 300000,
      modifier: "skip", // this only works for tracing enabled nodes
      test: async function () {
        if (skipTest.skip) {
          log(
            "No test data available for " +
              `${skipTest.networkName} #${skipTest.chainId} , skipping test.`
          );
          return; // TODO: replace this with this.skip() when added to vitest
        }

        const promises = traceStatic.testData.map(async (a) => {
          try {
            const result = await limiter.schedule(() =>
              (context.ethers().provider as ethers.JsonRpcProvider).send("debug_traceTransaction", [
                a.txHash,
              ])
            );
            log(
              `Successful tracing response from runtime ${a.runtime} in block #${a.blockNumber}.`
            );
            return { runtime: a.runtime, blockNumber: a.blockNumber, error: false, result };
          } catch (e: any) {
            return { runtime: a.runtime, blockNumber: a.blockNumber, error: true, result: e };
          }
        });

        const results = await Promise.all(promises.flat());
        const failures = results.filter((a) => {
          if (a.error === true) {
            log(
              `Failure tracing in runtime ${a.runtime}, blocknumber ${a.blockNumber} ` +
                `: ${a.result}`
            );
            return true;
          }
        });
        expect(failures).to.be.empty;
      },
    });

    it({
      id: "C200",
      title: "can call eth_getTransactionReceipt",
      timeout: 300000,
      test: async function () {
        if (skipTest.skip) {
          log(
            "No test data available for " +
              `${skipTest.networkName} #${skipTest.chainId}` +
              " , skipping test."
          );
          return; // TODO: replace this with this.skip() when added to vitest
        }

        const promises = traceStatic.testData.map(async (a) => {
          try {
            const result = await limiter.schedule(() =>
              (context.ethers().provider as ethers.JsonRpcProvider).send(
                "eth_getTransactionReceipt",
                [a.txHash]
              )
            );
            log(`Successful response from runtime ${a.runtime} in block #${a.blockNumber}.`);
            const error = result == null;
            return { runtime: a.runtime, blockNumber: a.blockNumber, error, result };
          } catch (e: any) {
            return { runtime: a.runtime, blockNumber: a.blockNumber, error: true, result: e };
          }
        });

        const results = await Promise.all(promises.flat());
        const failures = results.filter((a) => {
          if (a.error === true) {
            log(
              `Failure fetching txn receipt on runtime ${a.runtime}, blocknumber ${a.blockNumber}` +
                ` and result: ${JSON.stringify(a.result)}`
            );
            return true;
          }
        });
        expect(failures).to.be.empty;
      },
    });

    it({
      id: "C300",
      title: `can call eth_protocolVersion`,
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_protocolVersion",
          []
        );
        expect(result).to.be.greaterThan(0);
      },
    });

    it({
      id: "C400",
      title: "can call eth_syncing",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_syncing",
          []
        );
        expect(result).to.satisfy((s: any) => typeof s === "number" || typeof s === "boolean");
      },
    });

    it({
      id: "C500",
      title: "can call eth_hashrate",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_hashrate",
          []
        );
        expect(result).to.contain("0x0");
      },
    });

    it({
      id: "C600",
      title: "can call eth_coinbase",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_coinbase",
          []
        );
        expect(result.length).to.equal(42);
      },
    });

    it({
      id: "C700",
      title: "can call eth_mining",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_mining",
          []
        );
        expect(result).to.equal(
          !!(await paraApi.rpc.system.nodeRoles()).find((role) => role.isAuthority)
        );
      },
    });

    it({
      id: "C800",
      title: "can call eth_chainId",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_chainId",
          []
        );
        expect(Number(result)).to.be.greaterThan(0);
      },
    });

    it({
      id: "C900",
      title: "can call eth_gasPrice",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_gasPrice",
          []
        );
        expect(Number(result)).to.be.greaterThan(0);
      },
    });

    it({
      id: "C1000",
      title: "can call eth_accounts",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_accounts",
          []
        );
        expect(result.length).to.be.greaterThanOrEqual(0);
      },
    });

    it({
      id: "C1100",
      title: "can call eth_blockNumber",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_blockNumber",
          []
        );
        expect(result.length).to.be.greaterThanOrEqual(0);
      },
    });

    it({
      id: "C1200",
      title: "can call eth_getBalance",
      test: async function () {
        const treasuryPalletId = paraApi.consts.treasury.palletId;
        const treasuryAddress = `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`;
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getBalance",
          [treasuryAddress, "latest"]
        );
        expect(BigInt(result) === 0n).to.be.false;
      },
    });

    it({
      id: "C1300",
      title: "can call eth_getStorageAt",
      test: async function () {
        if (skipTest.skip) {
          log(
            "No test data available for" +
              `${skipTest.networkName} #${skipTest.chainId} , skipping test.`
          );
          return; // TODO: replace this with this.skip() when added to vitest
        }

        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getStorageAt",
          [traceStatic.WETH, "0x0", "latest"]
        );
        expect(BigInt(result) === 0n).to.be.false;
      },
    });

    it({
      id: "C1400",
      title: "can call eth_getBlockByHash",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getBlockByHash",
          [blockHash, false]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C1500",
      title: "can call eth_getBlockByNumber",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getBlockByNumber",
          ["latest", false]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C1600",
      title: "can call eth_getTransactionCount",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getTransactionCount",
          [collatorAddress, "latest"]
        );
        expect(Number(result)).to.be.greaterThanOrEqual(0);
      },
    });

    it({
      id: "C1700",
      title: "can call eth_getBlockTransactionCountByHash",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getBlockTransactionCountByHash",
          [blockHash]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C1800",
      title: "can call eth_getBlockTransactionCountByNumber",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getBlockTransactionCountByNumber",
          ["latest"]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C1900",
      title: "can call eth_getUncleCountByBlockHash",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getUncleCountByBlockHash",
          [blockHash]
        );
        expect(result).to.contain("0x0");
      },
    });

    it({
      id: "C2000",
      title: "can call eth_getCode",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getCode",
          [collatorAddress, "latest"]
        );
        expect(result).to.equal("0x");
      },
    });

    it({
      id: "C2100",
      title: "can call eth_estimateGas",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_estimateGas",
          [
            {
              from: collatorAddress,
              to: collatorAddress,
              value: "0x9184e72a",
              data:
                "0xd46e8dd67c5d32be8d46e8dd67c5d3" +
                "2be8058bb8eb970870f072445675058bb8eb970870f072445675",
            },
          ]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C2200",
      title: "can call eth_feeHistory",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_feeHistory",
          ["4", "latest", []]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C2300",
      title: "can call eth_getTransactionByBlockHashAndIndex",
      test: async function () {
        const block = (await paraApi.query.ethereum.currentBlock()).unwrap();
        if (block.transactions.length === 0) {
          log("No transactions in block, skipping test");
          return; // TODO: replace this with this.skip() when added to vitest
        }
        const number = block.header.number.toNumber();
        const hash = await paraApi.query.ethereum.blockHash(number);
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getTransactionByBlockHashAndIndex",
          [hash, 0]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C2400",
      title: `can call eth_getTransactionByBlockNumberAndIndex`,
      test: async function () {
        const block = (await paraApi.query.ethereum.currentBlock()).unwrap();
        if (block.transactions.length === 0) {
          log("No transactions in block, skipping test");
          return; // TODO: replace this with this.skip() when added to vitest
        }
        const number = block.header.number.toNumber();
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getTransactionByBlockNumberAndIndex",
          [number, 0]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C2500",
      title: `can call eth_getUncleByBlockHashAndIndex`,
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getUncleByBlockHashAndIndex",
          [blockHash, 0]
        );
        expect(result).to.be.null;
      },
    });

    it({
      id: "C2600",
      title: `can call eth_getUncleByBlockNumberAndIndex`,
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getUncleByBlockNumberAndIndex",
          [blockNumber, 0]
        );
        expect(result).to.be.null;
      },
    });

    it({
      id: "C2700",
      title: "can call eth_getLogs",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_getLogs",
          [{ blockHash }]
        );
        expect(result).to.not.be.null;
      },
    });

    it({
      id: "C2800",
      title: "can call eth_submitWork",
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_submitWork",
          [
            numberToHex(blockNumber + 1, 64),
            "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
            "0xD1FE5700000000000000000000000000D1FE5700000000000000000000000000",
          ]
        );
        expect(result).to.be.false;
      },
    });

    it({
      id: "C2900",
      title: `can call eth_submitHashrate`,
      test: async function () {
        const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
          "eth_submitHashrate",
          [
            "0x0000000000000000000000000000000000000000000000000000000000500000",
            "0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c",
          ]
        );
        expect(result).to.be.false;
      },
    });

    it({
      id: "C3000",
      title: "can call eth_newFilter",
      test: async function () {
        try {
          const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_newFilter",
            [{ fromBlock: "latest" }]
          );
          expect(result).to.not.be.null;
        } catch (e: any) {
          if (e.toString().includes("Error: Filter pool is full")) {
            log(`Filter pool is full, skipping test.`);
            return; // TODO: replace this with this.skip() when added to vitest
          }
          expect.fail(null, null, e.toString());
        }
      },
    });

    it({
      id: "C3100",
      title: "can call eth_newBlockFilter",
      test: async function () {
        try {
          const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_newBlockFilter",
            []
          );
          expect(result).to.not.be.null;
        } catch (e: any) {
          if (e.toString().includes("Error: Filter pool is full")) {
            log(`Filter pool is full, skipping test.`);
            return; // TODO: replace this with this.skip() when added to vitest
          }
          expect.fail(null, null, e.toString());
        }
      },
    });

    it({
      id: "C3200",
      title: "can call eth_getFilterChanges",
      test: async function () {
        try {
          const filterId = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_newFilter",
            [{ fromBlock: "latest" }]
          );
          const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_getFilterChanges",
            [filterId]
          );
          expect(result).to.not.be.null;
        } catch (e: any) {
          if (e.toString().includes("Error: Filter pool is full")) {
            log(`Filter pool is full, skipping test.`);
            return; // TODO: replace this with this.skip() when added to vitest
          }
          expect.fail(null, null, e.toString());
        }
      },
    });

    it({
      id: "C3300",
      title: "can call eth_getFilterLogs",
      test: async function () {
        try {
          const filterId = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_newFilter",
            [{ fromBlock: "latest" }]
          );
          const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_getFilterLogs",
            [filterId]
          );
          expect(result).to.not.be.null;
        } catch (e: any) {
          if (e.toString().includes("Error: Filter pool is full")) {
            log(`Filter pool is full, skipping test.`);
            return; // TODO: replace this with this.skip() when added to vitest
          }
          expect.fail(null, null, e.toString());
        }
      },
    });

    it({
      id: "C3400",
      title: "can call eth_uninstallFilter",
      test: async function () {
        try {
          const filterId = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_newFilter",
            [{ fromBlock: "latest" }]
          );
          const result = await (context.ethers().provider as ethers.JsonRpcProvider).send(
            "eth_uninstallFilter",
            [filterId]
          );
          expect(result).to.be.true;
        } catch (e: any) {
          if (e.toString().includes("Error: Filter pool is full")) {
            log(`Filter pool is full, skipping test.`);
            return; // TODO: replace this with this.skip() when added to vitest
          }
          expect.fail(null, null, e.toString());
        }
      },
    });
  },
});
