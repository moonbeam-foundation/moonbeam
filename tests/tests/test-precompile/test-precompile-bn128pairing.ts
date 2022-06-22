import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Precompiles - bn128Pairing", (context) => {
  it("should be accessible from a smart contract", async function () {
    const { rawTx } = await createContract(context, "Bn128Pairing");
    await context.createBlock(rawTx);

    // Because the call to bn128mul is in the constructor of HashRipmd160, verifying the code
    // is enough.
    expect(await context.web3.eth.getCode("0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3")).equals(
      "0x6080604052600080fdfea2646970667358221220e90355f07d7a4ae3a9df347abcddaab" +
        "722cb5be69464e1ff818d231c9ee0b8de64736f6c63430008030033"
    );
  });
});
