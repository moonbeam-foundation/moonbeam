import "@moonbeam-network/api-augment";
import { hexToBigInt, nToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { Contract } from "web3-eth-contract";
import { alith, baltathar } from "../../util/accounts";
import { GLMR } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectOk } from "../../util/expect";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const STATE_OVERRIDE_CONTRACT_JSON = getCompiled("StateOverrideTest");
const STATE_OVERRIDE_INTERFACE = new ethers.utils.Interface(
  STATE_OVERRIDE_CONTRACT_JSON.contract.abi
);

async function setupContract(context: DevTestContext) {
  const { contract, rawTx } = await createContract(
    context,
    "StateOverrideTest",
    {
      ...ALITH_TRANSACTION_TEMPLATE,
      value: Web3.utils.toWei("1", "ether"),
    },
    [100n]
  );
  await expectOk(context.createBlock(rawTx));
  return contract;
}

describeDevMoonbeam("Call - State Override", (context) => {
  let stateOverrideContract: Contract;
  let contractAddress: string;

  before("setup contract and set allowance", async function () {
    stateOverrideContract = await setupContract(context);
    contractAddress = stateOverrideContract.options.address;

    await expectOk(
      context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: contractAddress,
          data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("setAllowance", [
            baltathar.address,
            10,
          ]),
        })
      )
    );
  });

  it("should have a balance of > 100 GLMR without state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: baltathar.address,
        to: stateOverrideContract.options.address,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("getSenderBalance"),
      },
    ]);
    expect(hexToBigInt(result) > 100n * GLMR).to.be.true;
  });

  it("should have a balance of 50 GLMR with state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: baltathar.address,
        to: stateOverrideContract.options.address,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("getSenderBalance"),
      },
      "latest",
      {
        [baltathar.address]: {
          balance: nToHex(50n * GLMR),
        },
      },
    ]);
    expect(hexToBigInt(result)).to.equal(50n * GLMR);
  });

  it("should have availableFunds of 100 without state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: stateOverrideContract.options.address,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("availableFunds"),
      },
    ]);
    expect(hexToBigInt(result)).to.equal(100n);
  });

  it("should have availableFunds of 500 with state override", async function () {
    const availableFundsKey = Web3.utils.padLeft(Web3.utils.numberToHex(1), 64); // slot 1
    const newValue = Web3.utils.padLeft(Web3.utils.numberToHex(500), 64);

    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: stateOverrideContract.options.address,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("availableFunds"),
      },
      "latest",
      {
        [contractAddress]: {
          stateDiff: {
            [availableFundsKey]: newValue,
          },
        },
      },
    ]);
    expect(hexToBigInt(result)).to.equal(500n);
  });

  it("should have allowance of 10 without state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: contractAddress,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("allowance", [
          alith.address,
          baltathar.address,
        ]),
      },
    ]);
    expect(hexToBigInt(result)).to.equal(10n);
  });

  it("should have allowance of 50 with state override", async function () {
    const allowanceKey = Web3.utils.soliditySha3(
      {
        type: "uint256",
        value: baltathar.address,
      },
      {
        type: "uint256",
        value: Web3.utils.soliditySha3(
          {
            type: "uint256",
            value: alith.address,
          },
          {
            type: "uint256",
            value: "2", // slot 2
          }
        ),
      }
    );
    const newValue = Web3.utils.padLeft(Web3.utils.numberToHex(50), 64);

    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: contractAddress,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("allowance", [
          alith.address,
          baltathar.address,
        ]),
      },
      "latest",
      {
        [contractAddress]: {
          stateDiff: {
            [allowanceKey]: newValue,
          },
        },
      },
    ]);
    expect(hexToBigInt(result)).to.equal(50n);
  });

  it("should have allowance 50 but availableFunds 0 with full state override", async function () {
    const allowanceKey = Web3.utils.soliditySha3(
      {
        type: "uint256",
        value: baltathar.address,
      },
      {
        type: "uint256",
        value: Web3.utils.soliditySha3(
          {
            type: "uint256",
            value: alith.address,
          },
          {
            type: "uint256",
            value: "2", // slot 2
          }
        ),
      }
    );
    const newValue = Web3.utils.padLeft(Web3.utils.numberToHex(50), 64);

    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: contractAddress,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("allowance", [
          alith.address,
          baltathar.address,
        ]),
      },
      "latest",
      {
        [contractAddress]: {
          state: {
            [allowanceKey]: newValue,
          },
        },
      },
    ]);
    expect(hexToBigInt(result)).to.equal(50n);

    const { result: result2 } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: contractAddress,
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("availableFunds"),
      },
      "latest",
      {
        [contractAddress]: {
          state: {
            [allowanceKey]: newValue,
          },
        },
      },
    ]);
    expect(hexToBigInt(result2)).to.equal(0n);
  });

  it("should set MultiplyBy7 deployedBytecode with state override", async function () {
    const testContract = getCompiled("MultiplyBy7");
    const testContractInterface = new ethers.utils.Interface(testContract.contract.abi);

    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: contractAddress,
        data: testContractInterface.encodeFunctionData("multiply", [5]), // multiplies by 7
      },
      "latest",
      {
        [contractAddress]: {
          code: `0x${testContract.contract.evm.deployedBytecode.object}`,
        },
      },
    ]);
    expect(hexToBigInt(result)).to.equal(35n);
  });
});
