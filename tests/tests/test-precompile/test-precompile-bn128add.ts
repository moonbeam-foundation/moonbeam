import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeam("Precompiles - bn128add", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { rawTx } = await createContract(context.web3, "Bn128Addition");
    await context.createBlock({ transactions: [rawTx] });

    // Because the call to bn128add is in the constructor of HashRipmd160, verifying the code
    // is enough.
    expect(await context.web3.eth.getCode("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a")).equals(
      "0x6080604052600080fdfea2646970667358221220a18633c4ec2f5fd19918720cc9181bf5" +
        "3e954372785d2b34c64298c5275b4d5264736f6c63430008030033"
    );
  });
});
