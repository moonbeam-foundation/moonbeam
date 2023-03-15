import { PRECOMPILE_GMP_ADDRESS } from "../../util/constants";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransaction } from "../../util/transactions";
import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";

const GMP_CONTRACT_JSON = getCompiled("GmpPrecompile");
const GMP_INTERFACE = new ethers.utils.Interface(GMP_CONTRACT_JSON.contract.abi);

describeDevMoonbeam(`Test exisiting Wormhole Payload Precompile VAA`, (context) => {
  it("should send transaction", async function () {
    this.timeout(3600 * 1000);

    /**
      https://github.com/wormhole-foundation/wormhole/blob/main/ethereum/contracts/Messages.sol#L147
    **/
    const vaa_header =
      "0x01" + // version
      "00000000" + // guardian_set_index
      "01" + // signature count
      // SIGNATURE
      ("00" + // guardian index
        "d98b0684017cffa1a56f77f3408d570809c440b533b8c3ef2b259b44cbf93105" + // r
        "352d11ab862e7c33fec4e54d3ef10d5c8928c5c3b3de15bcb1d5d92db66a92f9" + // S
        "01"); // v

    const vaa_body =
      "640b665d" + // timestamp
      "00000001" + // nonce
      "000a" + // emitter chain
      "000000000000000000000000599cea2204b4faecd584ab1f2b6aca137a0afbe8" + // emitter address
      "000000000000009e" + // sequence
      "01" + // "consistency level"
      // PAYLOAD
      "030000000000000000000000000000000000000000000000000000000001312d" +
      "00000000000000000000000000f1277d1ed8ad466beddf92ef448a1326619566" +
      "21000a0000000000000000000000000000000000000000000000000000000000" +
      "0008150010000000000000000000000000b7e8c35609ca73277b2207d07b51c9" +
      "ac5798b380000000000000000000000000000000000000000000000000000000" +
      "0000000020000000000000000000000000000000000000000000000000000000" +
      "00000000807b2022706172656e7473223a20312c2022696e746572696f72223a" +
      "207b20225832223a205b207b202250617261636861696e223a20383838207d2c" +
      "207b20224163636f756e744b65793230223a2022307833353442313044343765" +
      "3834413030366239453765363641323239443137344538464632413036332220" +
      "7d205d7d7d";

    const vaa = vaa_header + vaa_body;

    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [vaa]);

    console.log("data:");
    console.log(data);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 500_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Stopped");
  });
});
