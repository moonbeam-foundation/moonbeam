import { expect } from "chai";
import { GENESIS_ACCOUNT } from "../util/constants";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

describeDevMoonbeam("Precompiles - sacrifice", (context) => {
  it("should be valid", async function () {
    const txCall = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: "0x01",
        to: "0x00000000000000000000000000000000000001FF",
        data: `0x0000000000005BA0`, // 23456
      },
    ]);

    console.log(txCall);

    // should return empty result
    expect(txCall.result).equals("0x");
  });
});

describeDevMoonbeam("Precompiles - sacrifice", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "SacrificeWrapper");
    await context.createBlock({ transactions: [rawTx] });

    // console.log("Contract => ", contract);

    foo = await contract.methods.sacrifice(12);
  });
});
