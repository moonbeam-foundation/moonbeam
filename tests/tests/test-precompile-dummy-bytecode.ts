import { expect } from "chai";
import { GENESIS_ACCOUNT } from "../util/constants";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

const DEPLOYED_BYTECODE = "0x60006000fd";

describeDevMoonbeam("Precompiles - dummy bytecodes ", (context) => {
  it("should return dummy bytecode for every precompiles ", async function () {
    [
      "0x0000000000000000000000000000000000000001",
      "0x0000000000000000000000000000000000000002",
      "0x0000000000000000000000000000000000000003",
      "0x0000000000000000000000000000000000000004",
      "0x0000000000000000000000000000000000000005",
      "0x0000000000000000000000000000000000000006",
      "0x0000000000000000000000000000000000000007",
      "0x0000000000000000000000000000000000000008",

      "0x0000000000000000000000000000000000000400",
      "0x0000000000000000000000000000000000000401",

      "0x0000000000000000000000000000000000000800",
    ].forEach(async (x) => {
      const code = await context.web3.eth.getCode(x);
      expect(code).to.equal(DEPLOYED_BYTECODE);
    });
  });
});
