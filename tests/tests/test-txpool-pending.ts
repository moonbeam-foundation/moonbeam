import { expect } from "chai";
import { Contract } from "web3-eth-contract";

import { GENESIS_ACCOUNT } from "../util/constants";
import { createContract, createContractExecution } from "../util/transactions";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";

describeDevMoonbeam("TxPool - Pending Ethereum transaction", (context) => {
  let txHash;
  before("Setup: Create transaction", async () => {
    const { rawTx } = await createContract(context.web3, "TestContract", {
      gas: 1048576,
    });
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
  });

  it("should appear in the txpool inspection", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    let data = inspect.result.pending[GENESIS_ACCOUNT.toLowerCase()][context.web3.utils.toHex(0)];
    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 1000000000 wei"
    );
  });

  it("should be marked as pending", async function () {
    const pendingTransaction = (
      await customWeb3Request(context.web3, "eth_getTransactionByHash", [txHash])
    ).result;
    // pending transactions do not know yet to which block they belong to
    expect(pendingTransaction).to.include({
      blockNumber: null,
      hash: txHash,
    });
  });

  it("should appear in the txpool content", async function () {
    let content = await customWeb3Request(context.web3, "txpool_content", []);

    const data = content.result.pending[GENESIS_ACCOUNT.toLowerCase()][context.web3.utils.toHex(0)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: GENESIS_ACCOUNT.toLowerCase(),
      gas: "0x100000",
      gasPrice: "0x3b9aca00",
      hash: txHash,
      nonce: context.web3.utils.toHex(0),
      to: "0x0000000000000000000000000000000000000000",
      value: "0x0",
    });
  });
});

describeDevMoonbeam("TxPool - Ethereum Contract Call", (context) => {
  let testContract: Contract, txHash;

  before("Setup: Create contract block and add call transaction", async () => {
    const { contract, rawTx } = await createContract(context.web3, "TestContract", {
      gas: 1048576,
    });
    testContract = contract;
    await context.createBlock({ transactions: [rawTx] });

    txHash = (
      await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        await createContractExecution(context.web3, {
          contract,
          contractCall: contract.methods.multiply(5),
        }),
      ])
    ).result;
  });

  it("should appear in the txpool inspection", async function () {
    const contractAddress = testContract.options.address;
    const inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    const data = inspect.result.pending[GENESIS_ACCOUNT.toLowerCase()][context.web3.utils.toHex(1)];

    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      contractAddress.toString().toLowerCase() + ": 0 wei + 12000000 gas x 1000000000 wei"
    );
  });

  it("should appear in the txpool content", async function () {
    const content = await customWeb3Request(context.web3, "txpool_content", []);
    const data = content.result.pending[GENESIS_ACCOUNT.toLowerCase()][context.web3.utils.toHex(1)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: GENESIS_ACCOUNT.toLowerCase(),
      gas: "0xb71b00",
      gasPrice: "0x3b9aca00",
      hash: txHash,
      nonce: context.web3.utils.toHex(1),
      to: testContract.options.address.toString().toLowerCase(),
      value: "0x0",
    });
  });
});
