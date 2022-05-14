import { expect } from "chai";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";
import { createContract, createTransfer } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeamAllEthTxTypes("Block Gas - Limit", (context) => {
  it("should be allowed to the max block gas", async function () {
    const { rawTx } = await createContract(context, "TestContract", {
      gas: EXTRINSIC_GAS_LIMIT,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    expect(txResults[0].result).to.not.be.null;

    const receipt = await context.web3.eth.getTransaction(txResults[0].result);
    expect(receipt.blockHash).to.be.not.null;
  });
});

describeDevMoonbeamAllEthTxTypes("Block Gas - Limit", (context) => {
  it("should fail setting it over the max block gas", async function () {
    const { rawTx } = await createContract(context, "TestContract", {
      gas: EXTRINSIC_GAS_LIMIT + 1,
    });

    expect(
      ((await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).error as any)
        .message
    ).to.equal("gas limit reached");
  });
});

describeDevMoonbeam("Block Gas - Limit", (context) => {
  // TODO: Joshy to fix block gas access in smart contract
  it.skip("should be accessible within a contract", async function () {
    const { contract, rawTx } = await createContract(context, "CheckBlockVariables");
    await context.createBlock({ transactions: [rawTx] });

    expect((await contract.methods.gaslimit().call()) !== "0").to.eq(true);
  });
});

describeDevMoonbeam("Block Gas - fill block with balance transfers", (context) => {
  it("should fill block with balance transfers", async function () {
    this.timeout(20000);
    const testAccount = "0x1111111111111111111111111111111111111111";
    console.log("EXTRINSIC_GAS_LIMIT: ", EXTRINSIC_GAS_LIMIT);
    let numTransfers = Math.floor(EXTRINSIC_GAS_LIMIT / 21000); // 618.8095238095239
    console.log("maximum number of balance transfers: ", numTransfers);

    // precondition: testAccount should have 0 balance
    expect(await context.web3.eth.getBalance(testAccount, 0)).to.equal("0");

    let transactions = [];
    for (let i = 0; i < numTransfers; i++) {
      let txn = await createTransfer(context, testAccount, 1, { nonce: i });
      transactions.push(txn);
    }

    console.log("num txns pushed: ", transactions.length);

    let start = Date.now();
    await context.createBlock({ transactions });
    let elapsed = Date.now() - start;
    console.log("block took: " + elapsed);

    expect(await context.web3.eth.getBalance(testAccount, 1)).to.equal("" + numTransfers);
  });
});
