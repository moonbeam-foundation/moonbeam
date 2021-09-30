import { expect } from "chai";
import { Contract } from "web3-eth-contract";

import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  EXTRINSIC_GAS_LIMIT,
} from "../util/constants";
import { createContract, createContractExecution, createTransaction } from "../util/transactions";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("TxPool - Future Ethereum transaction", (context) => {
  let txHash;
  before("Setup: Create transaction", async () => {
    const { rawTx } = await createContract(context.web3, "TestContract", {
      gas: 1048576,
      nonce: 1, // future nonce
    });
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
  });

  it("should appear in the txpool inspection", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    let data = inspect.result.queued[GENESIS_ACCOUNT.toLowerCase()][context.web3.utils.toHex(1)];
    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 1000000000 wei"
    );
  });

  it("should appear in the txpool content", async function () {
    let content = await customWeb3Request(context.web3, "txpool_content", []);

    const data = content.result.queued[GENESIS_ACCOUNT.toLowerCase()][context.web3.utils.toHex(1)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: GENESIS_ACCOUNT.toLowerCase(),
      gas: "0x100000",
      gasPrice: "0x3b9aca00",
      hash: txHash,
      nonce: context.web3.utils.toHex(1),
      to: "0x0000000000000000000000000000000000000000",
      value: "0x0",
    });
  });
});

describeDevMoonbeam("TxPool - Skipped transaction should unpool dependent ones", (context) => {
  it("skipped transaction should unpool dependent ones", async function () {
    const limit = EXTRINSIC_GAS_LIMIT - 30_000; // let's leave a bit of room

    const { rawTx: deployTx, contract } = await createContract(context.web3, "InfiniteContract", {
      gas: 1048576,
      nonce: 0,
      gasPrice: "0x2540BE400",
    });

    // 1. transaction using some good chunk of block weight
    const setupTx = await createContractExecution(
      context.web3,
      {
        contract,
        contractCall: contract.methods.infinite(),
      },
      {
        from: GENESIS_ACCOUNT,
        privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
        value: "0x0",
        gas: `0x${limit.toString(16)}`,
        gasPrice: "0x2540BE401",
        nonce: 1,
      }
    );

    // 2. transaction being to big to fit in the same block as previous tx
    const tooBigTx = await createTransaction(context.web3, {
      from: GENESIS_ACCOUNT,
      privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
      value: "0x0",
      gas: `0x${limit.toString(16)}`,
      gasPrice: "0x2540BE402",
      to: GENESIS_ACCOUNT,
      data: `0x0`,
      nonce: 2,
    });

    // 3. while this transaction seems valid in regard of nonces, the previous
    // tx is too big for the block and will be skipped, making this transaction
    // invalid. This transaction thus must not be included in the Substrate block.
    const invalidTx = await createTransaction(context.web3, {
      from: GENESIS_ACCOUNT,
      privateKey: GENESIS_ACCOUNT_PRIVATE_KEY,
      value: "0x0",
      gas: "0x10000", // small enough to go throught
      gasPrice: "0x2540BE403",
      to: GENESIS_ACCOUNT,
      data: `0x0`,
      nonce: 3,
    });

    await customWeb3Request(context.web3, "eth_sendRawTransaction", [deployTx]);
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [setupTx]);
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [tooBigTx]);
    await customWeb3Request(context.web3, "eth_sendRawTransaction", [invalidTx]);

    const blockReceipt = await context.createBlock();

    const block = await context.polkadotApi.rpc.chain.getBlock(blockReceipt.block.hash);

    // 3 inherents + contract deployement + heavy call
    // if there is 6, it means that either transction with nonce 2 or 3 have been included while
    // they shouldn't !
    expect(block.block.extrinsics.length).to.be.eq(5);

    console.log(JSON.stringify(block));
  });
});
