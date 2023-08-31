import { expect } from "chai";

import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Transaction Cost discards", (context) => {
  it("should take transaction cost into account and not submit it to the pool", async function () {
    // This is a contract deployment signed by Alith but that doesn't have a high enough
    // gaslimit. Since web3 prevents to sign transactions that cannot pay its tx cost we
    // had build it and sign it manually.
    const tx = await customWeb3Request(context.web3, "eth_sendRawTransaction", [
      "0xf9011b80843b9aca008252088080b8c960806040526000805534801561001457600080fd5b5060005b60648\
      1101561003557806000819055508080600101915050610018565b506085806100446000396000f3fe608060405\
      2348015600f57600080fd5b506004361060285760003560e01c80631572821714602d575b600080fd5b6033604\
      9565b6040518082815260200191505060405180910390f35b6000548156fea264697066735822122015105f2e5\
      f98d0c6e61fe09f704e2a86dd1cbf55424720229297a0fff65fe04064736f6c63430007000033820a26a08ac98\
      ea04dec8017ebddd1e87cc108d1df1ef1bf69ba35606efad4df2dfdbae2a07ac9edffaa0fd7c91fa5688b5e36a\
      1944944bca22b8ff367e4094be21f7d85a3",
    ]);
    let msg = "intrinsic gas too low";
    expect(tx.error).to.include({
      message: msg,
    });
  });
});
