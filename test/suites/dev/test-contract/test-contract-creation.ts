import "@moonbeam-network/api-augment";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import {
  alith,
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  baltathar,
  EXTRINSIC_GAS_LIMIT,
  GLMR,
  MIN_GAS_PRICE,
} from "@moonwall/util";
import { expectTypeOf } from "vitest";
import { PrivateKeyAccount, WalletClient, encodeDeployData, keccak256, toRlp } from "viem";
import { privateKeyToAccount, generatePrivateKey } from "viem/accounts";
import {
  TransactionTypes,
  deployAndCreateCompiledContract,
  sendRawTransaction,
  createRawTransaction,
} from "../../../helpers/viem.js";
import { getCompiled } from "../../../helpers/contracts.js";
import { call } from "node_modules/viem/dist/types/actions/public/call.js";
import { hexToU8a, numberToHex, numberToU8a, u8aToHex, u8aToString } from "@polkadot/util";
import { stringToU8a } from "@polkadot/util";
import { sendTransaction } from "viem/wallet";
import * as RLP from "rlp";
import { keccakAsHex } from "@polkadot/util-crypto";
import { ethers } from "ethers";
import { verifyLatestBlockFees } from "../../../helpers/block.js";

// TODO: expand these tests to do multiple txn types when added to viem
describeSuite({
  id: "D0601",
  title: "Contract creation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    for (const txnType of TransactionTypes) {
      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 1}`,
        title: `should return the ${txnType} transaction hash`,
        test: async function () {
          const { hash } = await deployAndCreateCompiledContract(context, "MultiplyBy7");
          expect(hash).toBeTruthy();
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 2}`,
        title: `${txnType} should return the contract code`,
        test: async () => {
          const contractData = getCompiled("MultiplyBy7");
          const callCode = (
            await context.viemClient("public").call({ data: contractData.byteCode })
          ).data;
          const { contractAddress } = await deployAndCreateCompiledContract(context, "MultiplyBy7");
          const deployedCode = await context
            .viemClient("public")
            .getBytecode({ address: contractAddress! });
          expect(callCode).to.be.eq(deployedCode);
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 3}`,
        title: `should not contain ${txnType}  contract at genesis`,
        test: async function () {
          const { contractAddress } = await deployAndCreateCompiledContract(context, "MultiplyBy7");
          expect(
            await context
              .viemClient("public")
              .getBytecode({ address: contractAddress!, blockNumber: 0n })
          ).toBeUndefined();
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 4}`,
        title: `${txnType} deployed contracts should store the code on chain`,
        test: async function () {
          // This is to enable pending tag support
          await context.createBlock();
          const code =
            "0x608060405234801561005d5760405162461bcd60e51b815260206004820152602260248201527f4574686572" +
            "2073656e7420746f206e6f6e2d70617961626c652066756e637469604482019081526137b760f11b6064830152" +
            "608482fd5b50600436106100785760003560e01c8063c6888fa1146100dd575b60405162461bcd60e51b815260" +
            "206004820152603560248201527f436f6e747261637420646f6573206e6f7420686176652066616c6c6261636b" +
            "2060448201908152746e6f7220726563656976652066756e6374696f6e7360581b6064830152608482fd5b6100" +
            "f06100eb366004610115565b610102565b60405190815260200160405180910390f35b600061010f8260076101" +
            "79565b92915050565b6000602082840312156101725760405162461bcd60e51b81526020600482015260226024" +
            "8201527f414249206465636f64696e673a207475706c65206461746120746f6f2073686f6044820152611c9d60" +
            "f21b6064820152608481fd5b5035919050565b808202811582820484141761010f57634e487b7160e01b600052" +
            "601160045260246000fdfea26469706673582212201908894ace7c2455a9a9c3f237348fbb18e18147a95c2fd7" +
            "096a971132e2f57f64736f6c63430008130033";

          const compiled = getCompiled("MultiplyBy7");
          const callData = encodeDeployData({
            abi: compiled.contract.abi,
            bytecode: compiled.byteCode,
            args: [],
          }) as `0x${string}`;

          const nonce = await context
            .viemClient("public")
            .getTransactionCount({ address: ALITH_ADDRESS });

          await context.viemClient("wallet").sendTransaction({ data: callData, nonce });

          const contractAddress = ("0x" +
            keccak256(RLP.encode([ALITH_ADDRESS, nonce]))
              .slice(12)
              .substring(14)) as `0x${string}`;

          expect(
            await context
              .viemClient("public")
              .getBytecode({ address: contractAddress, blockTag: "pending" })
          ).to.deep.equal(code);

          await context.createBlock();

          expect(
            await context
              .viemClient("public")
              .getBytecode({ address: contractAddress, blockTag: "latest" })
          ).to.deep.equal(code);
        },
      });

      it({
        id: `T0${TransactionTypes.indexOf(txnType) + 5}`,
        title: `should check latest block fees for ${txnType}`,
        test: async function () {
          await context.createBlock();
          await deployAndCreateCompiledContract(context, "Fibonacci", {maxPriorityFeePerGas: 0n});
          await verifyLatestBlockFees(context);
        },
      });
    }
  },
});
