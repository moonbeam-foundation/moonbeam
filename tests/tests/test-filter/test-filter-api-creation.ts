import { expect } from "chai";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Filter API", (context) => {
  it("should be able to create a Log filter", async function () {
    const { contract, rawTx } = await createContract(context, "SingleEventContract");
    await context.createBlock({ transactions: [rawTx] });

    const createFilter = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: [
          "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
          "0x5c4242beB94dE30b922f57241f1D02f36e906915",
        ],
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(createFilter.result).to.be.eq(context.web3.utils.numberToHex(1));
  });
});

describeDevMoonbeamAllEthTxTypes("Filter API - Creating", (context) => {
  it("should increment filter id", async function () {
    const { contract, rawTx } = await createContract(context, "SingleEventContract");
    await context.createBlock({ transactions: [rawTx] });

    const createFilter = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x1",
        toBlock: "0x2",
        address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(createFilter.result).to.be.eq(context.web3.utils.numberToHex(1));

    const createFilter2 = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x1",
        toBlock: "0x2",
        address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(createFilter2.result).to.be.eq(context.web3.utils.numberToHex(2));
  });
});

describeDevMoonbeam("Filter Block API - Creating", (context) => {
  it("should be able to create a Block Log filter", async function () {
    const createFilter = await customWeb3Request(context.web3, "eth_newBlockFilter", []);
    expect(createFilter.result).to.be.eq(context.web3.utils.numberToHex(1));
  });
});
