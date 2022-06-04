import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Precompiles - ripemd160 ", (context) => {
  it("should be valid", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: "0x0000000000000000000000000000000000000003",
          data: `0x${Buffer.from("Hello world!").toString("hex")}`,
        })
      ).result
    ).equals("0x0000000000000000000000007f772647d88750add82d8e1a7a3e5c0902a346a3");
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - ripemd160 ", (context) => {
  it("should be accessible from a smart contract", async function () {
    await context.createBlock({
      transactions: [(await createContract(context, "HashRipmd160")).rawTx],
    });

    // Because the call to ripemd160 is in the constructor of HashRipmd160, verifying the code
    // is enough
    expect(await context.web3.eth.getCode("0xc01ee7f10ea4af4673cfff62710e1d7792aba8f3")).equals(
      "0x6080604052600080fdfea26469706673582212202a18a661fdf5ea3600714f19a16e1681d5c651e" +
        "3b23f5a55166c1372b7f4119b64736f6c63430008030033"
    );
  });
});
