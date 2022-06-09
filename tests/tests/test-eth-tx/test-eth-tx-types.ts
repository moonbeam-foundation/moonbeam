import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { baltathar } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransfer } from "../../util/transactions";

describeDevMoonbeam(
  "Ethereum Transaction - Legacy",
  (context) => {
    it("should contain valid legacy Ethereum data", async function () {
      await context.createBlock(createTransfer(context, baltathar.address, 512));

      const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
      let extrinsic = signedBlock.block.extrinsics.find((ex) => ex.method.section == "ethereum")
        .args[0] as any;
      expect(extrinsic.isLegacy).to.be.true;
      expect(extrinsic.asLegacy.toJSON()).to.deep.equal({
        nonce: 0,
        gasPrice: 1000000000,
        gasLimit: 12000000,
        action: { call: baltathar.address.toLowerCase() },
        value: 512,
        input: "0x",
        signature: {
          v: 2597,
          r: "0x440c713c1ea8ced9edacac8a33baa89411dca31af33b5c6e2c8e4a3c95112ab4",
          s: "0x17c303f32862b65034da593cc0fb1178c915ef7a0b9c221ff3b7d00647b208fb",
        },
      });
    });
  },
  "Legacy",
  false
);

describeDevMoonbeam(
  "Ethereum Transaction - EIP2930",
  (context) => {
    it("should contain valid EIP2930 Ethereum data", async function () {
      // Accesslist mock data, it doesn't matter.
      await context.createBlock(
        createTransfer(context, baltathar.address, 512, {
          accessList: [
            [
              "0x0000000000000000000000000000000000000000",
              ["0x0000000000000000000000000000000000000000000000000000000000000000"],
            ],
          ],
        })
      );

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
          call: baltathar.address.toLowerCase(),
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
        r: "0xb18ae6b035dfdf47954130cc9fa74ce051a59500d569b78a9b5e30d97e821682",
        s: "0x3fc8aa94a1e068a9463b3ece33efc98a199e3f72f15cb9707a133425e9448cb1",
      });
    });
  },
  "EIP2930",
  false
);

describeDevMoonbeam(
  "Ethereum Transaction - EIP1559",
  (context) => {
    it("should contain valid EIP1559 Ethereum data", async function () {
      // Accesslist mock data, it doesn't matter.
      await context.createBlock(
        createTransfer(context, baltathar.address, 512, {
          accessList: [
            [
              "0x0000000000000000000000000000000000000000",
              ["0x0000000000000000000000000000000000000000000000000000000000000000"],
            ],
          ],
        })
      );

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
          call: baltathar.address.toLowerCase(),
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
        r: "0xf631af2a9504e6a87764310814c00a9260ba328da571008d60f4f770f43bee5d",
        s: "0x016d1aaa3dcff84a35e7fcf5947738de786fdbac8270c438710bda917fdcb96f",
      });
    });
  },
  "EIP1559",
  false
);
