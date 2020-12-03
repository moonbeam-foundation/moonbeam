import { expect } from "chai";

import { TransactionReceipt } from "web3-core";

import {
  callContractFunctionMS,
  createAndFinalizeBlock,
  customRequest,
  deployContractManualSeal,
  describeWithMoonbeam,
} from "./util";
import {
  FINITE_LOOP_CONTRACT_ABI,
  FINITE_LOOP_CONTRACT_BYTECODE,
  FIRST_CONTRACT_ADDRESS,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  INFINITE_CONTRACT_ABI,
  INFINITE_CONTRACT_ABI_VAR,
  INFINITE_CONTRACT_BYTECODE,
  INFINITE_CONTRACT_BYTECODE_VAR,
  TEST_CONTRACT_ABI,
  TEST_CONTRACT_BYTECODE,
  TEST_CONTRACT_BYTECODE_INCR,
  TEST_CONTRACT_INCR_ABI,
} from "./constants";

describeWithMoonbeam("Moonbeam RPC (Contract Methods)", `simple-specs.json`, (context) => {
  before("create the contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.web3);
  });

  it("get transaction by hash", async () => {
    const latestBlock = await context.web3.eth.getBlock("latest");
    expect(latestBlock.transactions.length).to.equal(1);

    const tx_hash = latestBlock.transactions[0];
    const tx = await context.web3.eth.getTransaction(tx_hash);
    expect(tx.hash).to.equal(tx_hash);
  });

  it("should return contract method result", async function () {
    const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
      gasPrice: "0x01",
    });

    expect(await contract.methods.multiply(3).call()).to.equal("21");
  });
  // Requires error handling
  it("should fail for missing parameters", async function () {
    const contract = new context.web3.eth.Contract(
      [{ ...TEST_CONTRACT_ABI, inputs: [] }],
      FIRST_CONTRACT_ADDRESS,
      {
        from: GENESIS_ACCOUNT,
        gasPrice: "0x01",
      }
    );
    await contract.methods
      .multiply()
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm revert: Reverted`
        )
      );
  });

  // Requires error handling
  it("should fail for too many parameters", async function () {
    const contract = new context.web3.eth.Contract(
      [
        {
          ...TEST_CONTRACT_ABI,
          inputs: [
            { internalType: "uint256", name: "a", type: "uint256" },
            { internalType: "uint256", name: "b", type: "uint256" },
          ],
        },
      ],
      FIRST_CONTRACT_ADDRESS,
      {
        from: GENESIS_ACCOUNT,
        gasPrice: "0x01",
      }
    );
    await contract.methods
      .multiply(3, 4)
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm revert: Reverted`
        )
      );
  });

  // Requires error handling
  it("should fail for invalid parameters", async function () {
    const contract = new context.web3.eth.Contract(
      [
        {
          ...TEST_CONTRACT_ABI,
          inputs: [
            {
              internalType: "address",
              name: "a",
              type: "address",
            },
          ],
        },
      ],
      FIRST_CONTRACT_ADDRESS,
      { from: GENESIS_ACCOUNT, gasPrice: "0x01" }
    );
    await contract.methods
      .multiply("0x0123456789012345678901234567890123456789")
      .call()
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: evm revert: Reverted`
        )
      );
  });
});
