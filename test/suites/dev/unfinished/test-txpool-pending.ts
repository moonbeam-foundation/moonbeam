import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { Contract } from "web3-eth-contract";

import { alith } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";
import { GLMR, MIN_GAS_PRICE } from "../../util/constants";

describeDevMoonbeam("TxPool - Pending Ethereum transaction", (context) => {
  let txHash: string;
  before("Setup: Create transaction", async () => {
    const { rawTx } = await createContract(context, "MultiplyBy7", {
      gas: 1048576,
    });
    txHash = (await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).result;
  });

  it("should appear in the txpool inspection", async function () {
    let inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    // web3 rpc returns lowercase
    let data = inspect.result.pending[alith.address.toLowerCase()][context.web3.utils.toHex(0)];
    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 10000000000 wei"
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
    // web3 rpc returns lowercase
    const data = content.result.pending[alith.address.toLowerCase()][context.web3.utils.toHex(0)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: alith.address.toLowerCase(),
      gas: "0x100000",
      gasPrice: "0x2540be400",
      hash: txHash,
      nonce: context.web3.utils.toHex(0),
      to: "0x0000000000000000000000000000000000000000",
      value: "0x0",
    });
  });
});

describeDevMoonbeam("TxPool - Ethereum Contract Call", (context) => {
  let multiplyBy7Contract: Contract;
  let txHash: string;

  before("Setup: Create contract block and add call transaction", async () => {
    const { contract, rawTx } = await createContract(context, "MultiplyBy7", {
      gas: 1048576,
    });
    multiplyBy7Contract = contract;
    await context.createBlock(rawTx);

    txHash = (
      await customWeb3Request(context.web3, "eth_sendRawTransaction", [
        await createContractExecution(
          context,
          {
            contract,
            contractCall: contract.methods.multiply(5),
          },
          { gas: 12000000, gasPrice: MIN_GAS_PRICE }
        ),
      ])
    ).result;
  });

  it("should appear in the txpool inspection", async function () {
    const contractAddress = multiplyBy7Contract.options.address;
    const inspect = await customWeb3Request(context.web3, "txpool_inspect", []);
    const data = inspect.result.pending[alith.address.toLowerCase()][context.web3.utils.toHex(1)];

    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      contractAddress.toLowerCase() + ": 0 wei + 12000000 gas x 10000000000 wei"
    );
  });

  it("should appear in the txpool content", async function () {
    const content = await customWeb3Request(context.web3, "txpool_content", []);
    const data = content.result.pending[alith.address.toLowerCase()][context.web3.utils.toHex(1)];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: alith.address.toLowerCase(),
      gas: "0xb71b00",
      gasPrice: "0x2540be400",
      hash: txHash,
      nonce: context.web3.utils.toHex(1),
      to: multiplyBy7Contract.options.address.toLowerCase(),
      value: "0x0",
    });
  });
});
