import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

[
  {
    loop: 1,
    gas: 44211,
  },
  {
    loop: 500,
    gas: 367076,
  },
  {
    loop: 600,
    gas: 431776,
  },
].forEach(({ loop, gas }) => {
  describeDevMoonbeamAllEthTxTypes("Contract loop", (context) => {
    it(`should consume ${gas} for ${loop} loop`, async function () {
      const { contract, rawTx } = await createContract(context, "Looper");
      await context.createBlock(rawTx);
      await context.createBlock(
        createContractExecution(context, {
          contract,
          contractCall: contract.methods.incrementalLoop(loop),
        })
      );

      expect(await contract.methods.count().call()).to.eq(loop.toString());

      const block = await context.web3.eth.getBlock("latest");
      expect(block.gasUsed).to.eq(gas);
    });
  });
});
