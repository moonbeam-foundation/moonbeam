import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { ALITH_PRIVATE_KEY, baltathar } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createTransaction, createTransfer } from "../../util/transactions";

describeDevMoonbeam(
  "Ethereum Transaction - Legacy",
  (context) => {
    it("should contain valid legacy Ethereum data", async function () {
      await context.createBlock(
        createTransaction(context, {
          privateKey: ALITH_PRIVATE_KEY,
          to: baltathar.address,
          gas: 12_000_000,
          gasPrice: 1_000_000_000,
          value: 512,
        })
      );

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
  "moonbase",
  false
);

describeDevMoonbeam(
  "Ethereum Transaction - EIP2930",
  (context) => {
    it("should contain valid EIP2930 Ethereum data", async function () {
      // Accesslist mock data, it doesn't matter.
      await context.createBlock(
        createTransfer(context, baltathar.address, 512, {
          accessList: [],
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
        gasLimit: 21000,
        action: {
          call: baltathar.address.toLowerCase(),
        },
        value: 512,
        input: "0x",
        accessList: [],
        oddYParity: false,
        r: "0x6d61b9498e1ddcfa490ef3cb45d0152ad328f7f61d69e61d901a21eab86814c0",
        s: "0x716b528435345a640bd31a94e699b10440e418ff0edf62a2874091a682459084",
      });
    });
  },
  "EIP2930",
  "moonbase",
  false
);

describeDevMoonbeam(
  "Ethereum Transaction - EIP1559",
  (context) => {
    it("should contain valid EIP1559 Ethereum data", async function () {
      // Accesslist mock data, it doesn't matter.
      await context.createBlock(
        createTransfer(context, baltathar.address, 512, {
          accessList: [],
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
        gasLimit: 21000,
        action: {
          call: baltathar.address.toLowerCase(),
        },
        value: 512,
        input: "0x",
        accessList: [],
        oddYParity: false,
        r: "0xff6a476d2d8d7b0a23fcb3f1471d1ddd4dec7f3803db7f769aa1ce2575e493ac",
        s: "0x4ebec202edd10edfcee87927090105b95d8b58f80550cdf4eda20327f3377ca6",
      });
    });
  },
  "EIP1559",
  "moonbase",
  false
);
