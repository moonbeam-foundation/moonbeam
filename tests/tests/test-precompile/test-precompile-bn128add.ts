import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Precompiles - bn128add", (context) => {
  it("should be accessible from a smart contract", async function () {
    await context.createBlock({
      transactions: [(await createContract(context, "Bn128Addition")).rawTx],
    });

    // Because the call to bn128add is in the constructor of HashRipmd160, verifying the code
    // is enough.
    expect(await context.web3.eth.getCode("0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3")).equals(
      "0x6080604052600080fdfea2646970667358221220a18633c4ec2f5fd19918720cc9181bf5" +
        "3e954372785d2b34c64298c5275b4d5264736f6c63430008030033"
    );
  });
});
