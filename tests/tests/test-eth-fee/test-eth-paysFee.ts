import "@moonbeam-network/api-augment";

import { expect } from "chai";
import web3 from "web3";

import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";
import { extractInfo } from "../../util/substrate-rpc";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

// We use ethers library in this test as apparently web3js's types are not fully EIP-1559
// compliant yet.
describeDevMoonbeamAllEthTxTypes("Ethereum - PaysFee", (context) => {
  it("should be false for successful ethereum transactions", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        value: web3.utils.toWei("1", "ether"),
      })
    );
    const info = extractInfo(events);
    expect(info).to.not.be.empty;
    expect(info.paysFee.isYes, "Transaction should be marked as paysFees == no").to.be.false;
  });
});
