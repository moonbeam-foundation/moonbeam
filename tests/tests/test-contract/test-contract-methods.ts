import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { Contract } from "web3-eth-contract";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Contract creation", (context) => {
  let multiplyBy7: Contract;
  let testContractTx: string;

  before("Setup: Create the contract", async function () {
    const { contract, rawTx } = await createContract(context, "MultiplyBy7");
    const { result } = await context.createBlock(rawTx);
    multiplyBy7 = contract;
    testContractTx = result.hash;
  });

  it("should appear in the block transaction list", async () => {
    const block = await context.web3.eth.getBlock(1);
    const txHash = block.transactions[0];
    expect(txHash).to.equal(testContractTx);
  });

  it("should be in the transaction list", async () => {
    const tx = await context.web3.eth.getTransaction(testContractTx);
    expect(tx.hash).to.equal(testContractTx);
  });

  it("should provide callable methods", async function () {
    expect(await multiplyBy7.methods.multiply(3).call()).to.equal("21");
  });

  // TODO: when web3 supports eip1559 and eip2930, this test should be adapted
  it("should fail for call method with missing parameters", async function () {
    // Create a fake contract based on origin deployed contract.
    // It make the multiply method supposed to have 0 arguments
    const contract = new context.web3.eth.Contract(
      [{ ...multiplyBy7.options.jsonInterface[0], inputs: [] }],
      multiplyBy7.options.address
    );
    await contract.methods
      .multiply()
      .call()
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err: { message: string }) =>
        expect(err.message).to.equal(
          `Returned error: VM Exception while processing transaction: revert`
        )
      );
  });

  // Requires error handling
  it("should fail for too many parameters", async function () {
    // Create a fake contract based on origin deployed contract.
    // It make the multiply method supposed to have 2 arguments
    const contract = new context.web3.eth.Contract(
      [
        {
          ...multiplyBy7.options.jsonInterface[0],
          inputs: [
            { internalType: "uint256", name: "a", type: "uint256" },
            { internalType: "uint256", name: "b", type: "uint256" },
          ],
        },
      ],
      multiplyBy7.options.address
    );

    await contract.methods
      .multiply(3, 4)
      .call()
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err: { message: string }) =>
        expect(err.message).to.equal(
          `Returned error: VM Exception while processing transaction: revert`
        )
      );
  });

  it("should fail for invalid parameters", async function () {
    // Create a fake contract based on origin deployed contract.
    // It make the multiply method supposed to have a address type argument
    const contract = new context.web3.eth.Contract(
      [
        {
          ...multiplyBy7.options.jsonInterface[0],
          inputs: [
            {
              internalType: "address",
              name: "a",
              type: "address",
            },
          ],
        },
      ],
      multiplyBy7.options.address
    );
    await contract.methods
      .multiply("0x0123456789012345678901234567890123456789")
      .call()
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err: { message: string }) =>
        expect(err.message).to.equal(
          `Returned error: VM Exception while processing transaction: revert`
        )
      );
  });
});
