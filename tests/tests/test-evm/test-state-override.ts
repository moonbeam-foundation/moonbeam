import "@moonbeam-network/api-augment";
import { hexToBigInt, hexToBn, hexToNumber, nToHex, numberToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { TransactionReceipt } from "web3-core";
import { Contract } from "web3-eth-contract";
import {
  alith,
  ALITH_ADDRESS,
  ALITH_GENESIS_FREE_BALANCE,
  ALITH_PRIVATE_KEY,
  baltathar,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  charleth,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import {
  CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
  CONTRACT_RANDOMNESS_STATUS_PENDING,
  DEFAULT_GENESIS_BALANCE,
  GLMR,
  MILLIGLMR,
  PRECOMPILE_RANDOMNESS_ADDRESS,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { expectOk } from "../../util/expect";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";

const STATE_OVERRIDE_CONTRACT_JSON = getCompiled("StateOverrideTest");
const STATE_OVERRIDE_INTERFACE = new ethers.utils.Interface(
  STATE_OVERRIDE_CONTRACT_JSON.contract.abi
);

const STATE_OVERRIDE_INITIAL_AMOUNT = 100n;

async function setupContract(context: DevTestContext) {
  const { contract, rawTx } = await createContract(
    context,
    "StateOverrideTest",
    {
      ...ALITH_TRANSACTION_TEMPLATE,
      value: Web3.utils.toWei("1", "ether"),
    },
    [STATE_OVERRIDE_INITIAL_AMOUNT]
  );
  await expectOk(context.createBlock(rawTx));
  return contract;
}

describeDevMoonbeam("State Override", (context) => {
  let stateOverrideContract: Contract;
  before("setup contract", async function () {
    stateOverrideContract = await setupContract(context);
  });

  it("should have a initial funds 0 without state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: stateOverrideContract.options.address,
        gas: "0x100000",
        value: "0x0",
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("availableFunds"),
      },
    ]);
    expect(hexToBigInt(result)).to.equal(STATE_OVERRIDE_INITIAL_AMOUNT);
  });

  it("should have a balance of 1 GLMR without state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: stateOverrideContract.options.address,
        gas: "0x100000",
        value: "0x0",
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("getBalance"),
      },
    ]);
    expect(hexToBigInt(result)).to.equal(1n * GLMR);
  });

  it("should have a balance of 10 GLMR with state override", async function () {
    const { result } = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: stateOverrideContract.options.address,
        gas: "0x100000",
        value: "0x0",
        data: STATE_OVERRIDE_INTERFACE.encodeFunctionData("getBalance"),
      },
      "latest",
      {
        balance: nToHex(50n * GLMR),
      },
    ]);
    expect(hexToBigInt(result)).to.equal(50n * GLMR);
  });
});
