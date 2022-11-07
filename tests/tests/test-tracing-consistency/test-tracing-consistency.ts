import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeamAllEthTxTypes, describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { tracingTxns } from "../../util/tracing-txns";

const debug = require("debug")("test-tracing:tracing-consistency");

describeDevMoonbeam(`Verifying tracing consistency...`, (context) => {
  before("Loading tracing static data", async function () {
  
    const chainId = (await context.polkadotApi.query.ethereumChainId.chainId()).toString();
    debug(`Running tracing tests against chainId ${chainId}.`)
    if (!Object.keys(tracingTxns).includes(chainId)) {
      debug(`ChainId ${chainId} not supported, skipping test.`);
      this.skip();
    }
    const traceStatic = tracingTxns[chainId];
  });

  it("timbo test", async function () {

    debug(`Humble beginnings`)
  });
});




// describeDevMoonbeamAllEthTxTypes("Receipt - Revert", (context) => {
//   it("should generate a receipt", async function () {
//     const { rawTx } = await createContract(context, "FailingConstructor");
//     const { result } = await context.createBlock(rawTx);
//     const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

//     expect(receipt.status).to.be.false;
//     expect(receipt).to.include({
//       blockNumber: 1,
//       contractAddress: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
//       cumulativeGasUsed: 54600,
//       from: "0xf24ff3a9cf04c71dbc94d0b566f7a27b94566cac",
//       gasUsed: 54600,
//       to: null,
//       transactionHash: result.hash,
//       transactionIndex: 0,
//     });
//   });
// });
