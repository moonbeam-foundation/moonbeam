import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

[
  {
    loop: 1,
    gas: 42889,
  },
  {
    loop: 500,
    gas: 1064354,
  },
  {
    loop: 600,
    gas: 1269054,
  },
].forEach(({ loop, gas }) => {
  describeDevMoonbeam("Contract loop", (context) => {
    it(`should consume ${gas} for ${loop} loop`, async function () {
      const { contract, rawTx } = await createContract(context.web3, "FiniteLoopContract");
      await context.createBlock({ transactions: [rawTx] });
      await context.createBlock({
        transactions: [
          await createContractExecution(context.web3, {
            contract,
            contractCall: contract.methods.incr(loop),
          }),
        ],
      });

      expect(await contract.methods.count().call()).to.eq(loop.toString());

      const block = await context.web3.eth.getBlock("latest");
      expect(block.gasUsed).to.eq(gas);
    });
  });
});
