import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Precompiles - bn128mul", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { rawTx } = await createContract(context, "Bn128Multiply");
    await context.createBlock({ transactions: [rawTx] });

    // Because the call to bn128mul is in the constructor of HashRipmd160, verifying the code
    // is enough.
    expect(await context.web3.eth.getCode("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a")).equals(
      "0x6080604052600080fdfea26469706673582212209a97bc97d5e3a377e8298e3b3a72b24963" +
        "abb30bc27bb2266ae137b12aac8cc964736f6c63430008030033"
    );
  });
});
