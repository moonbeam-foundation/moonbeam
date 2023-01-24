import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { numberToHex } from "@polkadot/util";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { NetworkTestArtifact, tracingTxns } from "../util/tracing-txns";
import Bottleneck from "bottleneck";
import { BigNumber, providers } from "ethers";

const debug = require("debug")("smoke:historic-compatibility");
const limiter = new Bottleneck({ maxConcurrent: 10, minTime: 100 });
const httpEndpoint = process.env.HTTP_URL;

describeSmokeSuite("S1200", `Verifying historic compatibility`, async (context, testIt) => {
  let traceStatic: NetworkTestArtifact;
  let skipTest = { skip: false, networkName: "", chainId: "" };
  let blockNumber: number;
  let blockHash: string;
  let collatorAddress: string;

  before("Loading static data", async function () {
    const chainId = (await context.polkadotApi.query.ethereumChainId.chainId()).toString();
    debug(`Loading test data for chainId ${chainId}.`);
    traceStatic = tracingTxns.find((a) => a.chainId.toString() === chainId);
    const networkName = (await context.polkadotApi.rpc.system.chain()).toString();
    const latestBlockNumberToCheck = traceStatic
      ? Math.max(...traceStatic.testData.map((d) => d.blockNumber))
      : 0;
    blockNumber = (await context.polkadotApi.query.ethereum.currentBlock())
      .unwrap()
      .header.number.toNumber();
    if (
      !traceStatic ||
      networkName !== traceStatic.networkLabel ||
      latestBlockNumberToCheck > blockNumber
    ) {
      skipTest = { skip: true, networkName, chainId };
    }

    blockHash = (await context.polkadotApi.query.ethereum.blockHash(blockNumber)).toString();
    collatorAddress = (
      await context.polkadotApi.query.parachainStaking.selectedCandidates()
    )[0].toString();
  });

  testIt("C100", `can call debug_traceTransaction`, async function () {
    if (httpEndpoint == null || httpEndpoint == "") {
      debug(`No HTTP_URL provided, skipping test.`);
      this.skip();
    }

    if (skipTest.skip) {
      debug(
        `No test data available for ${skipTest.networkName} #${skipTest.chainId} , skipping test.`
      );
      this.skip();
    }

    this.timeout(300000);
    const provider = new providers.JsonRpcProvider(httpEndpoint);
    const promises = traceStatic.testData.map(async (a) => {
      try {
        const result = await limiter.schedule(() =>
          provider.send("debug_traceTransaction", [a.txHash])
        );
        debug(`Successful tracing response from runtime ${a.runtime} in block #${a.blockNumber}.`);
        return { runtime: a.runtime, blockNumber: a.blockNumber, error: false, result };
      } catch (e) {
        return { runtime: a.runtime, blockNumber: a.blockNumber, error: true, result: e };
      }
    });

    const results = await Promise.all(promises.flatMap((a) => a));
    const failures = results.filter((a) => {
      if (a.error === true) {
        debug(
          `Failure tracing in runtime ${a.runtime}, blocknumber ${a.blockNumber} ` + `: ${a.result}`
        );
        return true;
      }
    });
    expect(failures).to.be.empty;
  });

  testIt("C200", `can call eth_getTransactionReceipt`, async function () {
    this.timeout(300000);

    if (skipTest.skip) {
      debug(
        `No test data available for ${skipTest.networkName} #${skipTest.chainId} , skipping test.`
      );
      this.skip();
    }

    const promises = traceStatic.testData.map(async (a) => {
      try {
        const result = await limiter.schedule(() =>
          context.ethers.send("eth_getTransactionReceipt", [a.txHash])
        );
        debug(`Successful response from runtime ${a.runtime} in block #${a.blockNumber}.`);
        const error = result == null;
        return { runtime: a.runtime, blockNumber: a.blockNumber, error, result };
      } catch (e) {
        return { runtime: a.runtime, blockNumber: a.blockNumber, error: true, result: e };
      }
    });

    const results = await Promise.all(promises.flatMap((a) => a));
    const failures = results.filter((a) => {
      if (a.error === true) {
        debug(
          `Failure fetching txn receipt on runtime ${a.runtime}, blocknumber ${a.blockNumber}` +
            ` and result: ${JSON.stringify(a.result)}`
        );
        return true;
      }
    });
    expect(failures).to.be.empty;
  });

  testIt("C300", `can call eth_protocolVersion`, async function () {
    const result = await context.ethers.send("eth_protocolVersion", []);
    expect(result).to.be.greaterThan(0);
  });

  testIt("C400", `can call eth_syncing`, async function () {
    const result = await context.ethers.send("eth_syncing", []);
    expect(result).to.satisfy((s) => typeof s == "number" || typeof s == "boolean");
  });

  testIt("C500", `can call eth_hashrate`, async function () {
    const result = await context.ethers.send("eth_hashrate", []);
    expect(result).to.contain("0x0");
  });

  testIt("C600", `can call eth_coinbase`, async function () {
    const result = await context.ethers.send("eth_coinbase", []);
    expect(result.length).to.equal(42);
  });

  testIt("C700", `can call eth_mining`, async function () {
    const result = await context.ethers.send("eth_mining", []);
    expect(result).to.equal(
      !!(await context.polkadotApi.rpc.system.nodeRoles()).find((role) => role.isAuthority)
    );
  });

  testIt("C800", `can call eth_chainId`, async function () {
    const result = await context.ethers.send("eth_chainId", []);
    expect(Number(result)).to.be.greaterThan(0);
  });

  testIt("C900", `can call eth_gasPrice`, async function () {
    const result = await context.ethers.send("eth_gasPrice", []);
    expect(Number(result)).to.be.greaterThan(0);
  });

  testIt("C1000", `can call eth_accounts`, async function () {
    const result = await context.ethers.send("eth_accounts", []);
    expect(result.length).to.be.greaterThanOrEqual(0);
  });

  testIt("C1100", `can call eth_blockNumber`, async function () {
    const result = await context.ethers.send("eth_blockNumber", []);
    expect(result.length).to.be.greaterThanOrEqual(0);
  });

  testIt("C1200", `can call eth_getBalance`, async function () {
    const treasuryPalletId = context.polkadotApi.consts.treasury.palletId;
    const treasuryAddress = `0x6d6f646C${treasuryPalletId.toString().slice(2)}0000000000000000`;
    const result = await context.ethers.send("eth_getBalance", [treasuryAddress, "latest"]);
    expect(BigNumber.from(result).isZero()).to.be.false;
  });

  testIt("C1300", `can call eth_getStorageAt`, async function () {
    if (skipTest.skip) {
      debug(
        `No test data available for ${skipTest.networkName} #${skipTest.chainId} , skipping test.`
      );
      this.skip();
    }

    const result = await context.ethers.send("eth_getStorageAt", [
      traceStatic.WETH,
      "0x0",
      "latest",
    ]);
    expect(BigNumber.from(result).isZero()).to.be.false;
  });

  testIt("C1400", `can call eth_getBlockByHash`, async function () {
    const result = await context.ethers.send("eth_getBlockByHash", [blockHash, false]);
    expect(result).to.not.be.null;
  });

  testIt("C1500", `can call eth_getBlockByNumber`, async function () {
    const result = await context.ethers.send("eth_getBlockByNumber", ["latest", false]);
    expect(result).to.not.be.null;
  });

  testIt("C1600", `can call eth_getTransactionCount`, async function () {
    const result = await context.ethers.send("eth_getTransactionCount", [
      collatorAddress,
      "latest",
    ]);
    expect(Number(result)).to.be.greaterThanOrEqual(0);
  });

  testIt("C1700", `can call eth_getBlockTransactionCountByHash`, async function () {
    const result = await context.ethers.send("eth_getBlockTransactionCountByHash", [blockHash]);
    expect(result).to.not.be.null;
  });

  testIt("C1800", `can call eth_getBlockTransactionCountByNumber`, async function () {
    const result = await context.ethers.send("eth_getBlockTransactionCountByNumber", ["latest"]);
    expect(result).to.not.be.null;
  });

  testIt("C1900", `can call eth_getUncleCountByBlockHash`, async function () {
    const result = await context.ethers.send("eth_getUncleCountByBlockHash", [blockHash]);
    expect(result).to.contain("0x0");
  });

  testIt("C2000", `can call eth_getCode`, async function () {
    const result = await context.ethers.send("eth_getCode", [collatorAddress, "latest"]);
    expect(result).to.equal("0x");
  });

  testIt("C2100", `can call eth_estimateGas`, async function () {
    const result = await context.ethers.send("eth_estimateGas", [
      {
        from: collatorAddress,
        to: collatorAddress,
        value: "0x9184e72a",
        data:
          "0xd46e8dd67c5d32be8d46e8dd67c5d3" +
          "2be8058bb8eb970870f072445675058bb8eb970870f072445675",
      },
    ]);
    expect(result).to.not.be.null;
  });

  testIt("C2200", `can call eth_feeHistory`, async function () {
    const result = await context.ethers.send("eth_feeHistory", ["4", "latest", []]);
    expect(result).to.not.be.null;
  });

  testIt("C2300", `can call eth_getTransactionByBlockHashAndIndex`, async function () {
    const block = (await context.polkadotApi.query.ethereum.currentBlock()).unwrap();
    if (block.transactions.length === 0) {
      debug("No transactions in block, skipping test");
      this.skip();
    }
    const number = block.header.number.toNumber();
    const hash = await context.polkadotApi.query.ethereum.blockHash(number);
    const result = await context.ethers.send("eth_getTransactionByBlockHashAndIndex", [hash, 0]);
    expect(result).to.not.be.null;
  });

  testIt("C2400", `can call eth_getTransactionByBlockNumberAndIndex`, async function () {
    const block = (await context.polkadotApi.query.ethereum.currentBlock()).unwrap();
    if (block.transactions.length === 0) {
      debug("No transactions in block, skipping test");
      this.skip();
    }
    const number = block.header.number.toNumber();
    const result = await context.ethers.send("eth_getTransactionByBlockNumberAndIndex", [
      number,
      0,
    ]);
    expect(result).to.not.be.null;
  });

  testIt("C2500", `can call eth_getUncleByBlockHashAndIndex`, async function () {
    const result = await context.ethers.send("eth_getUncleByBlockHashAndIndex", [blockHash, 0]);
    expect(result).to.be.null;
  });

  testIt("C2600", `can call eth_getUncleByBlockNumberAndIndex`, async function () {
    const result = await context.ethers.send("eth_getUncleByBlockNumberAndIndex", [blockNumber, 0]);
    expect(result).to.be.null;
  });

  testIt("C2700", `can call eth_getLogs`, async function () {
    const result = await context.ethers.send("eth_getLogs", [{ blockHash }]);
    expect(result).to.not.be.null;
  });

  testIt("C2800", `can call eth_submitWork`, async function () {
    const result = await context.ethers.send("eth_submitWork", [
      numberToHex(blockNumber + 1, 64),
      "0x1234567890abcdef1234567890abcdef1234567890abcdef1234567890abcdef",
      "0xD1FE5700000000000000000000000000D1FE5700000000000000000000000000",
    ]);
    expect(result).to.be.false;
  });

  testIt("C2900", `can call eth_submitHashrate`, async function () {
    const result = await context.ethers.send("eth_submitHashrate", [
      "0x0000000000000000000000000000000000000000000000000000000000500000",
      "0x59daa26581d0acd1fce254fb7e85952f4c09d0915afd33d3886cd914bc7d283c",
    ]);
    expect(result).to.be.false;
  });

  testIt("C3000", `can call eth_newFilter`, async function () {
    try {
      const result = await context.ethers.send("eth_newFilter", [{ fromBlock: "latest" }]);
      expect(result).to.not.be.null;
    } catch (e) {
      if (e.toString().includes("Error: Filter pool is full")) {
        debug(`Filter pool is full, skipping test.`);
        this.skip();
      } else {
        expect.fail(null, null, e.toString());
      }
    }
  });

  testIt("C3100", `can call eth_newBlockFilter`, async function () {
    try {
      const result = await context.ethers.send("eth_newBlockFilter", []);
      expect(result).to.not.be.null;
    } catch (e) {
      if (e.toString().includes("Error: Filter pool is full")) {
        debug(`Filter pool is full, skipping test.`);
        this.skip();
      } else {
        expect.fail(null, null, e.toString());
      }
    }
  });

  testIt("C3200", `can call eth_getFilterChanges`, async function () {
    try {
      const filterId = await context.ethers.send("eth_newFilter", [{ fromBlock: "latest" }]);
      const result = await context.ethers.send("eth_getFilterChanges", [filterId]);
      expect(result).to.not.be.null;
    } catch (e) {
      if (e.toString().includes("Error: Filter pool is full")) {
        debug(`Filter pool is full, skipping test.`);
        this.skip();
      } else {
        expect.fail(null, null, e.toString());
      }
    }
  });

  testIt("C3300", `can call eth_getFilterLogs`, async function () {
    try {
      const filterId = await context.ethers.send("eth_newFilter", [{ fromBlock: "latest" }]);
      const result = await context.ethers.send("eth_getFilterLogs", [filterId]);
      expect(result).to.not.be.null;
    } catch (e) {
      if (e.toString().includes("Error: Filter pool is full")) {
        debug(`Filter pool is full, skipping test.`);
        this.skip();
      } else {
        expect.fail(null, null, e.toString());
      }
    }
  });

  testIt("C3400", `can call eth_uninstallFilter`, async function () {
    try {
      const filterId = await context.ethers.send("eth_newFilter", [{ fromBlock: "latest" }]);
      const result = await context.ethers.send("eth_uninstallFilter", [filterId]);
      expect(result).to.be.true;
    } catch (e) {
      if (e.toString().includes("Error: Filter pool is full")) {
        debug(`Filter pool is full, skipping test.`);
        this.skip();
      } else {
        expect.fail(null, null, e.toString());
      }
    }
  });
});
