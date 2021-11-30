import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract, createContractExecution } from "../util/transactions";

describeDevMoonbeam("Fibonacci", (context) => {
  it("should be able to call fibonacci", async function () {
    const { contract, rawTx } = await createContract(context, "Fibonacci");
    await context.createBlock({ transactions: [rawTx] });

    expect(await contract.methods.fib2(0).call()).to.equal("" + 0);
    expect(await contract.methods.fib2(1).call()).to.equal("" + 1);
    expect(await contract.methods.fib2(2).call()).to.equal("" + 1);
    expect(await contract.methods.fib2(3).call()).to.equal("" + 2);
    expect(await contract.methods.fib2(4).call()).to.equal("" + 3);
    expect(await contract.methods.fib2(5).call()).to.equal("" + 5);

    expect(await contract.methods.fib2(20).call()).to.equal("" + 6765);

    // the largest Fib number supportable by a uint256 is 370.
    // actual value: 94611056096305838013295371573764256526437182762229865607320618320601813254535
    expect(await contract.methods.fib2(370).call()).to.equal(
      "94611056096305838013295371573764256526437182762229865607320618320601813254535"
    );
  });

  it("should be able to call fibonacci[370] in txn", async function () {
    const { contract, rawTx } = await createContract(context, "Fibonacci");
    await context.createBlock({ transactions: [rawTx] });

    const tx = await createContractExecution(context, {
      contract,
      contractCall: contract.methods.fib2(370),
    });

    const { txResults } = await context.createBlock({ transactions: [tx] });
    const receipt = await context.web3.eth.getTransactionReceipt(txResults[0].result);
    expect(receipt.status).to.be.true;
  });
});
