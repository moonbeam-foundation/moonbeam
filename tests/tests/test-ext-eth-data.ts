import { expect } from "chai";
import { GENESIS_ACCOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer } from "../util/transactions";

describeDevMoonbeam(
  "Ethereum Extrinsic (Legacy)",
  (context) => {
    it("should contain valid legacy Ethereum data", async function () {
      const testAddress = "0x1111111111111111111111111111111111111111";
      await context.createBlock({
        transactions: [await createTransfer(context, testAddress, 512)],
      });

      const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
      let extrinsic = signedBlock.block.extrinsics.find((ex) => ex.method.section == "ethereum")
        .args[0] as any;
      expect(extrinsic.isLegacy).to.be.true;
      expect(extrinsic.asLegacy.toJSON()).to.deep.equal({
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
  },
  "Legacy",
  false
);

describeDevMoonbeam(
  "Ethereum Extrinsic (EIP2930)",
  (context) => {
    it("should contain valid EIP2930 Ethereum data", async function () {
      const testAddress = "0x1111111111111111111111111111111111111111";
      // Accesslist mock data, it doesn't matter.
      await context.createBlock({
        transactions: [
          await createTransfer(context, testAddress, 512, {
            accessList: [
              [
                "0x0000000000000000000000000000000000000000",
                ["0x0000000000000000000000000000000000000000000000000000000000000000"],
              ],
            ],
          }),
        ],
      });

      const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
      let extrinsic = signedBlock.block.extrinsics.find((ex) => ex.method.section == "ethereum")
        .args[0] as any;
      expect(extrinsic.isEip2930).to.be.true;
      expect(extrinsic.asEip2930.toJSON()).to.deep.equal({
        chainId: 1281,
        nonce: 0,
        gasPrice: 1000000000,
        gasLimit: 12000000,
        action: {
          call: "0x1111111111111111111111111111111111111111",
        },
        value: 512,
        input: "0x",
        accessList: [
          {
            address: "0x0000000000000000000000000000000000000000",
            storageKeys: ["0x0000000000000000000000000000000000000000000000000000000000000000"],
          },
        ],
        oddYParity: true,
        r: "0xb3afc47c1048d0a7d02bd90cfd90dffcdaa26fddc1644df23439b5ce94d19f1a",
        s: "0x5cfa40c0c59e5c67fd2dac5bc0934c8d7f8b9970c153c878e2c8a1f23c67a3b9",
      });
    });
  },
  "EIP2930",
  false
);

describeDevMoonbeam(
  "Ethereum Extrinsic (EIP1559)",
  (context) => {
    it("should contain valid EIP1559 Ethereum data", async function () {
      const testAddress = "0x1111111111111111111111111111111111111111";
      // Accesslist mock data, it doesn't matter.
      await context.createBlock({
        transactions: [
          await createTransfer(context, testAddress, 512, {
            accessList: [
              [
                "0x0000000000000000000000000000000000000000",
                ["0x0000000000000000000000000000000000000000000000000000000000000000"],
              ],
            ],
          }),
        ],
      });

      const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
      let extrinsic = signedBlock.block.extrinsics.find((ex) => ex.method.section == "ethereum")
        .args[0] as any;
      expect(extrinsic.isEip1559).to.be.true;
      expect(extrinsic.asEip1559.toJSON()).to.deep.equal({
        chainId: 1281,
        nonce: 0,
        maxPriorityFeePerGas: 0,
        maxFeePerGas: 1000000000,
        gasLimit: 12000000,
        action: {
          call: "0x1111111111111111111111111111111111111111",
        },
        value: 512,
        input: "0x",
        accessList: [
          {
            address: "0x0000000000000000000000000000000000000000",
            storageKeys: ["0x0000000000000000000000000000000000000000000000000000000000000000"],
          },
        ],
        oddYParity: false,
        r: "0x7477d1ec3db20e2726a69e7aab7e1b6beda2a312222e4db85c316cc796e655bf",
        s: "0x53b7ae2a82b3cebaaec6086620bcf683c5171d5669152280f56bdfc9e322f284",
      });
    });
  },
  "EIP1559",
  false
);
