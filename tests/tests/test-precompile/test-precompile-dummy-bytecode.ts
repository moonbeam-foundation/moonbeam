import "@moonbeam-network/api-augment";

import { expect } from "chai";
import * as RLP from "rlp";

import { alith } from "../../util/accounts";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createTransaction } from "../../util/transactions";

const DEPLOYED_BYTECODE = "0x60006000fd";

// push1 5 (deployed bytecode length)
// dup1
// push1 11 (offset of deployed bytecode in this initcode)
// push1 0 (offset in target memory)
// codecopy (copy code slice into memory)
// push1 0 (offset in target memory)
// return
// <deployed bytecode>
const INIT_CODE = "0x600580600B6000396000F360006000fd";

describeDevMoonbeamAllEthTxTypes("Precompiles - precompiles dummy bytecode", (context) => {
  it("should return dummy bytecode for every precompiles", async function () {
    [
      "0x0000000000000000000000000000000000000001",
      "0x0000000000000000000000000000000000000002",
      "0x0000000000000000000000000000000000000003",
      "0x0000000000000000000000000000000000000004",
      "0x0000000000000000000000000000000000000005",
      "0x0000000000000000000000000000000000000006",
      "0x0000000000000000000000000000000000000007",
      "0x0000000000000000000000000000000000000008",

      "0x0000000000000000000000000000000000000400",
      "0x0000000000000000000000000000000000000401",
      "0x0000000000000000000000000000000000000402",

      "0x0000000000000000000000000000000000000800",
    ].forEach(async (x) => {
      const code = await context.web3.eth.getCode(x);
      expect(code).to.equal(DEPLOYED_BYTECODE);
    });
  });

  it("should revert when dummy bytecode is called", async function () {
    // we deploy a new contract with the same bytecode to be able to
    // execute the bytecode instead of executing a precompile.
    await context.createBlock(
      createTransaction(context, {
        data: INIT_CODE,
      })
    );

    const contractAddress =
      "0x" +
      context.web3.utils
        .sha3(RLP.encode([alith.address, 0]) as any)
        .slice(12)
        .substring(14);

    // check the deployed code by this init code watches what we use for precompiles.
    const code = await context.web3.eth.getCode(contractAddress);
    expect(code).to.equal(DEPLOYED_BYTECODE);

    // try to call contract (with empty data, shouldn't matter)

    const { result } = await context.createBlock(
      createTransaction(context, {
        gas: 12000000,
        data: "0x",
        to: contractAddress,
      })
    );
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

    expect(receipt.status).to.equal(false);
    // 21006 = call cost + 2*PUSH cost
    expect(receipt.gasUsed).to.equal(21006);
  });
});
