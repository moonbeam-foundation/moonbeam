import { expect } from "chai";
import { GENESIS_ACCOUNT } from "../../util/constants";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Precompiles - ripemd160 ", (context) => {
  it("should be valid", async function () {
    const txCall = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000003",
        data: `0x${Buffer.from("Hello world!").toString("hex")}`,
      },
    ]);

    expect(txCall.result).equals(
      "0x0000000000000000000000007f772647d88750add82d8e1a7a3e5c0902a346a3"
    );
  });
});

describeDevMoonbeam("Precompiles - ripemd160 ", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { contract, rawTx } = await createContract(context, "HashRipmd160");
    await context.createBlock({ transactions: [rawTx] });

    // Because the call to ripemd160 is in the constructor of HashRipmd160, verifying the code
    // is enough
    expect(await context.web3.eth.getCode("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a")).equals(
      "0x6080604052600080fdfea26469706673582212202a18a661fdf5ea3600714f19a16e1681d5c651e" +
        "3b23f5a55166c1372b7f4119b64736f6c63430008030033"
    );
  });
});
