import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { Contract } from "web3-eth-contract";

describeDevMoonbeam("Contract creation", (context) => {
  let testContract: Contract;
  let testContractTx: string;

  before("Setup: Create the contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "TestContract");
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    testContract = contract;
    testContractTx = txResults[0].result;
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
    expect(await testContract.methods.multiply(3).call()).to.equal("21");
  });

  it("should fail for call method with missing parameters", async function () {
    // Create a fake contract based on origin deployed contract.
    // It make the multiply method supposed to have 0 arguments
    const contract = new context.web3.eth.Contract(
      [{ ...testContract.options.jsonInterface[0], inputs: [] }],
      testContract.options.address
    );
    await contract.methods
      .multiply()
      .call()
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err) =>
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
          ...testContract.options.jsonInterface[0],
          inputs: [
            { internalType: "uint256", name: "a", type: "uint256" },
            { internalType: "uint256", name: "b", type: "uint256" },
          ],
        },
      ],
      testContract.options.address
    );

    await contract.methods
      .multiply(3, 4)
      .call()
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err) =>
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
          ...testContract.options.jsonInterface[0],
          inputs: [
            {
              internalType: "address",
              name: "a",
              type: "address",
            },
          ],
        },
      ],
      testContract.options.address
    );
    await contract.methods
      .multiply("0x0123456789012345678901234567890123456789")
      .call()
      .then(() => {
        return Promise.reject({ message: "Execution succeeded but should have failed" });
      })
      .catch((err) =>
        expect(err.message).to.equal(
          `Returned error: VM Exception while processing transaction: revert`
        )
      );
  });
});
