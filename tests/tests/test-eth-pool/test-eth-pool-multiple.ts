import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { Transaction } from "web3-core";

import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

/*
  At rpc-level, there is no interface for retrieving emulated pending transactions - emulated
    transactions that exist in the Substrate's pending transaction pool. Instead they are added to a
    shared collection (Mutex) with get/set locking to serve requests that ask for this transactions
    information before they are included in a block.
    We want to test that:
      - We resolve multiple promises in parallel that will write in this collection on the rpc-side
      - We resolve multiple promises in parallel that will read from this collection on the rpc-side
      - We can get the final transaction data once it leaves the pending collection
  */
describeDevMoonbeamAllEthTxTypes("EthPool - Multiple pending transactions", (context) => {
  let txHashes: string[];

  before("Setup: Sending 10 transactions", async function () {
    txHashes = await Promise.all(
      new Array(10).fill(0).map(async (_, i) => {
        const { rawTx } = await createContract(context, "MultiplyBy7", { nonce: i });
        return (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
      })
    );
  });

  it("should all be available by hash", async function () {
    const txs = (
      await Promise.all(
        txHashes.map((txHash) => {
          return customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash]);
        })
      )
    ).map((response) => response.result as Transaction);

    expect(txs).to.be.lengthOf(10);
    for (let i = 0; i < 10; i++) {
      expect(txs[i].hash).to.be.equal(txHashes[i]);
    }
  });

  it("should all be marked as pending", async function () {
    const txs = (
      await Promise.all(
        txHashes.map((txHash) => {
          return customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash]);
        })
      )
    ).map((response) => response.result as Transaction);

    expect(txs).to.be.lengthOf(10);
    for (let i = 0; i < 10; i++) {
      expect(txs[i].blockNumber).to.be.null;
      expect(txs[i].transactionIndex).to.be.null;
    }
  });
});

describeDevMoonbeamAllEthTxTypes("EthPool - Multiple produced transactions", (context) => {
  let txHashes: string[];

  before("Setup: Sending 10 transactions", async function () {
    txHashes = await Promise.all(
      new Array(10).fill(0).map(async (_, i) => {
        const { rawTx } = await createContract(context, "MultiplyBy7", { nonce: i });
        return (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
      })
    );
    // Put all the transaction in a produced block
    await context.createBlock();
  });

  it("should all be available by hash", async function () {
    const txs = (
      await Promise.all(
        txHashes.map((txHash) => {
          return customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash]);
        })
      )
    ).map((response) => response.result as Transaction);

    expect(txs).to.be.lengthOf(10);
    for (let i = 0; i < 10; i++) {
      expect(txs[i].hash).to.be.equal(txHashes[i]);
    }
  });

  it("should all have assigned block number", async function () {
    const txs = (
      await Promise.all(
        txHashes.map((txHash) => {
          return customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash]);
        })
      )
    ).map((response) => response.result as Transaction);

    expect(txs).to.be.lengthOf(10);
    for (let i = 0; i < 10; i++) {
      expect(txs[i].blockNumber).to.not.be.null;
    }
  });

  it("should all have a valid transactionIndex", async function () {
    const txs = (
      await Promise.all(
        txHashes.map((txHash) => {
          return customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash]);
        })
      )
    ).map((response) => response.result as Transaction);

    expect(txs).to.be.lengthOf(10);
    for (let i = 0; i < 10; i++) {
      expect(txs[i].hash).to.be.equal(txHashes[i]);
      expect(BigInt(txs[i].transactionIndex)).to.equal(BigInt(i));
    }
  });
});
