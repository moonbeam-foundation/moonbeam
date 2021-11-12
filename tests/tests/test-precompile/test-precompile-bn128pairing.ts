import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Precompiles - bn128Pairing", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { rawTx } = await createContract(context.web3, "Bn128Pairing");
    await context.createBlock({ transactions: [rawTx] });

    // Because the call to bn128mul is in the constructor of HashRipmd160, verifying the code
    // is enough.
    expect(await context.web3.eth.getCode("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a")).equals(
      "0x6080604052600080fdfea2646970667358221220e90355f07d7a4ae3a9df347abcddaab" +
        "722cb5be69464e1ff818d231c9ee0b8de64736f6c63430008030033"
    );
  });
});
