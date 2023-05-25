import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { expectOk } from "../../../helpers/expect.js";
import {
  GLMR,
  alith,
  baltathar,
  createEthersTxn,
  deployCreateCompiledContract,
} from "@moonwall/util";
import { encodeFunctionData, encodePacked, keccak256, pad, parseEther } from "viem";
import { Abi } from "abitype";
import { hexToBigInt, nToHex } from "@polkadot/util";
import { customDevRpcRequest } from "../../../helpers/common.js";
import { getCompiled } from "../../../helpers/contracts.js";
import Web3 from "web3";

describeSuite({
  id: "D0901",
  title: "Call - State Override",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let stateOverrideAddress: string;
    let contractAbi: Abi;

    beforeAll(async function () {
      const { contract, contractAddress, abi, status } = await deployCreateCompiledContract(
        context,
        "StateOverrideTest",
        { args: [100n], value: parseEther("1") }
      );

      expect(status).to.equal("success");

      const { rawSigned } = await createEthersTxn(context, {
        to: contractAddress,
        data: encodeFunctionData({
          abi,
          functionName: "setAllowance",
          args: [baltathar.address, 10n],
        }),
        gasLimit: 10_000_000,
      });

      await expectOk(context.createBlock(rawSigned));

      contractAbi = abi;
      stateOverrideAddress = contractAddress;
    });

    it({
      id: "T01",
      title: "should have a balance of > 100 GLMR without state override",
      test: async function () {
        const { data } = await context.viemClient("public").call({
          account: baltathar.address,
          to: stateOverrideAddress as `0x${string}`,
          data: encodeFunctionData({ abi: contractAbi, functionName: "getSenderBalance" }),
        });
        expect(hexToBigInt(data) > 100n * GLMR).to.be.true;
      },
    });

    it({
      id: "T02",
      title: "should have a balance of 50 GLMR with state override",
      test: async function () {
        const result = await customDevRpcRequest("eth_call", [
          {
            from: baltathar.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({ abi: contractAbi, functionName: "getSenderBalance" }),
          },
          "latest",
          {
            [baltathar.address]: {
              balance: nToHex(50n * GLMR),
            },
          },
        ]);

        expect(hexToBigInt(result)).to.equal(50n * GLMR);
      },
    });

    it({
      id: "T03",
      title: "should have availableFunds of 100 without state override",
      test: async function () {
        const result = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({ abi: contractAbi, functionName: "availableFunds" }),
          },
        ]);
        expect(hexToBigInt(result)).to.equal(100n);
      },
    });

    it({
      id: "T04",
      title: "should have availableFunds of 500 with state override",
      test: async function () {
        const availableFundsKey = pad(nToHex(1)); // slot 1
        const newValue = pad(nToHex(500));

        const result = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({ abi: contractAbi, functionName: "availableFunds" }),
          },
          "latest",
          {
            [stateOverrideAddress]: {
              stateDiff: {
                [availableFundsKey]: newValue,
              },
            },
          },
        ]);

        expect(hexToBigInt(result)).to.equal(500n);
      },
    });

    it({
      id: "T05",
      title: "should have allowance of 10 without state override",
      test: async function () {
        const result = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({
              abi: contractAbi,
              functionName: "allowance",
              args: [alith.address, baltathar.address],
            }),
          },
        ]);
        expect(hexToBigInt(result)).to.equal(10n);
      },
    });

    it({
      id: "T06",
      title: "should have allowance of 50 with state override",
      test: async function () {
        const allowanceKey = keccak256(
          encodePacked(
            ["uint256", "uint256"],
            [
              baltathar.address,
              keccak256(
                encodePacked(
                  ["uint256", "uint256"],
                  [
                    alith.address,
                    2n, // slot 2
                  ]
                )
              ) as unknown as bigint,
            ]
          )
        );

        const newValue = pad(nToHex(50));
        const result = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({
              abi: contractAbi,
              functionName: "allowance",
              args: [alith.address, baltathar.address],
            }),
          },
          "latest",
          {
            [stateOverrideAddress]: {
              stateDiff: {
                [allowanceKey]: newValue,
              },
            },
          },
        ]);
        expect(hexToBigInt(result)).to.equal(50n);
      },
    });

    it({
      id: "T07",
      title: "should have allowance 50 but availableFunds 0 with full state override",
      test: async function () {
        const allowanceKey = keccak256(
          encodePacked(
            ["uint256", "uint256"],
            [
              baltathar.address,
              keccak256(
                encodePacked(
                  ["uint256", "uint256"],
                  [
                    alith.address,
                    2n, // slot 2
                  ]
                )
              ) as unknown as bigint,
            ]
          )
        );
        const newValue = pad(nToHex(50));
        const result = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({
              abi: contractAbi,
              functionName: "allowance",
              args: [alith.address, baltathar.address],
            }),
          },
          "latest",
          {
            [stateOverrideAddress]: {
              state: {
                [allowanceKey]: newValue,
              },
            },
          },
        ]);
        expect(hexToBigInt(result)).to.equal(50n);

        const result2 = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({
              abi: contractAbi,
              functionName: "availableFunds",
            }),
          },
          "latest",
          {
            [stateOverrideAddress]: {
              state: {
                [allowanceKey]: newValue,
              },
            },
          },
        ]);
        expect(hexToBigInt(result2)).to.equal(0n);
      },
    });

    it({
      id: "T08",
      title: "should set MultiplyBy7 deployedBytecode with state override",
      test: async function () {
        const {
          contract: { abi: multiAbi, evm },
        } = getCompiled("MultiplyBy7");
        const result = await customDevRpcRequest("eth_call", [
          {
            from: alith.address,
            to: stateOverrideAddress,
            data: encodeFunctionData({
              abi: multiAbi,
              functionName: "multiply",
              args: [5n],
            }),
          },
          "latest",
          {
            [stateOverrideAddress]: {
              code: `0x${evm.deployedBytecode.object}`,
            },
          },
        ]);
        expect(hexToBigInt(result)).to.equal(35n);
      },
    });
  },
});
