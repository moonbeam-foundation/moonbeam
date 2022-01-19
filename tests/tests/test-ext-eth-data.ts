import { expect } from "chai";
import { GENESIS_ACCOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam("Ethereum Extrinsic", (context) => {
  it("should contain valid Ethereum data", async function () {
    const testAddress = "0x1111111111111111111111111111111111111111";
    await context.createBlock({
      transactions: [await createTransfer(context, testAddress, 512)],
    });

    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    expect(
      signedBlock.block.extrinsics.find((ex) => ex.method.section == "ethereum").args[0].toJSON()
    ).to.include({
      nonce: 0,
      gasPrice: 1000000000,
      gasLimit: 12000000,
      action: { call: "0x1111111111111111111111111111111111111111" },
      value: 512,
      input: "0x",
      signature: {
        v: 2598,
        r: "0x8c69faf613b9f72dbb029bb5d5acf42742d214c79743507e75fdc8adecdee928",
        s: "0x01be4f58ff278ac61125a81a582a717d9c5d6554326c01b878297c6522b12282",
      },
    });
  });
});
