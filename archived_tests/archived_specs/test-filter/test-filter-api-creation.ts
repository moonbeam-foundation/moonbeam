import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Filter API", (context) => {
  it("should be able to create a Log filter", async function () {
    const { rawTx } = await createContract(context, "EventEmitter");
    await context.createBlock(rawTx);

    const createFilter = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: [
          "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
          "0x970951a12F975E6762482ACA81E57D5A2A4e73F4",
        ],
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(createFilter.result).to.be.eq(context.web3.utils.numberToHex(1));
  });
});

describeDevMoonbeamAllEthTxTypes("Filter API - Creating", (context) => {
  it("should increment filter id", async function () {
    const { rawTx } = await createContract(context, "EventEmitter");
    await context.createBlock(rawTx);

    const createFilter = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x1",
        toBlock: "0x2",
        address: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(createFilter.result).to.be.eq(context.web3.utils.numberToHex(1));

    const createFilter2 = await customWeb3Request(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x1",
        toBlock: "0x2",
        address: "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
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
