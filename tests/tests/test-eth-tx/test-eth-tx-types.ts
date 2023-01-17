import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { ALITH_PRIVATE_KEY, baltathar } from "../../util/accounts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import {
  createTransaction,
  createTransfer,
  DEFAULT_TXN_MAX_BASE_FEE,
} from "../../util/transactions";

describeDevMoonbeam(
  "Ethereum Transaction - Legacy",
  (context) => {
    it("should contain valid legacy Ethereum data", async function () {
      await context.createBlock(
        createTransaction(context, {
          privateKey: ALITH_PRIVATE_KEY,
          to: baltathar.address,
          gas: 12_000_000,
          gasPrice: 10_000_000_000,
          value: 512,
        })
      );

      const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
      let extrinsic = signedBlock.block.extrinsics.find((ex) => ex.method.section == "ethereum")
        .args[0] as any;
      expect(extrinsic.isLegacy).to.be.true;
      expect(extrinsic.asLegacy.toJSON()).to.deep.equal({
        nonce: 0,
        gasPrice: DEFAULT_TXN_MAX_BASE_FEE,
        gasLimit: 12000000,
        action: { call: baltathar.address.toLowerCase() },
        value: 512,
        input: "0x",
        signature: {
          v: 2598,
          r: "0xc4d57ab7b0e601a95299b70a46fdbb16371b477669e0f8245e0a9f12e27e15f2",
          s: "0x4186b0a32dd279fed20b1f20805845e24eff2a2a035801fe19419c09e861a62d",
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
        gasPrice: DEFAULT_TXN_MAX_BASE_FEE,
        gasLimit: 21000,
        action: {
          call: baltathar.address.toLowerCase(),
        },
        value: 512,
        input: "0x",
        accessList: [],
        oddYParity: true,
        r: "0x28b384f1bf4b0ff05cf0d9002a9bdc2cfd20ee105f3dbdca737d59eded43785f",
        s: "0x73bcb4d0d6419becc9ee4db2ff80961443686b92d24c22a43f2e769cf080bbd8",
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
        maxFeePerGas: DEFAULT_TXN_MAX_BASE_FEE,
        gasLimit: 21000,
        action: {
          call: baltathar.address.toLowerCase(),
        },
        value: 512,
        input: "0x",
        accessList: [],
        oddYParity: false,
        r: "0x40f376b6ece87cedb35b8687bc50cfef91450a43af5b04b7d368c2164b9f100e",
        s: "0x76f61710d719e35878d022a90a278d8e291502314a46b33f83f03894a6b36871",
      });
    });
  },
  "EIP1559",
  "moonbase",
  false
);
