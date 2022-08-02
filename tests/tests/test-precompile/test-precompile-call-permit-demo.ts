import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import Web3 from "web3";
import { TransactionReceipt } from "web3-core";
import { Contract } from "web3-eth-contract";
import { signTypedData, SignTypedDataVersion, recoverTypedSignature } from "@metamask/eth-sig-util";
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
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";

const CALL_PERMIT_DEMO_CONTRACT_JSON = getCompiled("CallPermitDemo");
const CALL_PERMIT_DEMO_INTERFACE = new ethers.utils.Interface(
  CALL_PERMIT_DEMO_CONTRACT_JSON.contract.abi
);

const CALL_PERMIT_CONTRACT_JSON = getCompiled("CallPermit");
const CALL_PERMIT_INTERFACE = new ethers.utils.Interface(CALL_PERMIT_CONTRACT_JSON.contract.abi);

async function setupWithParticipants(context: DevTestContext) {
  const { contract, rawTx } = await createContract(context, "CallPermitDemo", {
    ...ALITH_TRANSACTION_TEMPLATE,
  });
  await context.createBlock(rawTx);
  return contract;
}

function getSignatureParameters(signature: string) {
  const r = signature.slice(0, 66);
  const s = `0x${signature.slice(66, 130)}`;
  let v = Web3.utils.toDecimal(`0x${signature.slice(130, 132)}`);

  if (![27, 28].includes(v)) v += 27;

  return {
    r,
    s,
    v,
  };
}

describeDevMoonbeam("Precompile - Call Permit - foo", (context) => {
  let demoContract: Contract;
  before("setup lottery contract", async function () {
    const { contract, rawTx } = await createContract(context, "CallPermitDemo", {
      ...ALITH_TRANSACTION_TEMPLATE,
    });
    await context.createBlock(rawTx);
    demoContract = contract;
  });

  it("should sign", async function () {
    const alithAccount = await context.polkadotApi.query.system.account(ALITH_ADDRESS);
    console.log(alithAccount.toHuman());
    const message = {
      from: ALITH_ADDRESS,
      to: CHARLETH_ADDRESS, // demoContract.options.address
      value: 42,
      data: "0x",
      gaslimit: 100000,
      nonce: alithAccount.nonce.toNumber(),
      deadline: 9999999999,
    };

    // const signature = context.web3.eth.accounts.sign(
    //   JSON.stringify(getMessageData(message)),
    //   ALITH_PRIVATE_KEY
    // );

    const data = {
      types: {
        EIP712Domain: [
          {
            name: "name",
            type: "string",
          },
          {
            name: "version",
            type: "string",
          },
          {
            name: "chainId",
            type: "uint256",
          },
          {
            name: "verifyingContract",
            type: "address",
          },
          // {
          //   name: "salt",
          //   type: "bytes32",
          // },
        ],
        CallPermit: [
          {
            name: "from",
            type: "address",
          },
          {
            name: "to",
            type: "address",
          },
          {
            name: "value",
            type: "uint256",
          },
          {
            name: "data",
            type: "bytes",
          },
          {
            name: "gaslimit",
            type: "uint64",
          },
          {
            name: "nonce",
            type: "uint256",
          },
          {
            name: "deadline",
            type: "uint256",
          },
        ],
      },
      primaryType: "CallPermit",
      domain: {
        name: "Call Permit Precompile",
        version: "1",
        chainId: 0,
        verifyingContract: "0x000000000000000000000000000000000000080a",
        // salt: Buffer.from([]),
      },
      message: message,
    };

    const signature = signTypedData({
      privateKey: Buffer.from(ALITH_PRIVATE_KEY.substring(2), "hex"),
      data: data as any,
      version: SignTypedDataVersion.V4,
    });
    // const signature = await context.web3.eth.sign(
    //   JSON.stringify(getMessageData(message)),
    //   ALITH_ADDRESS
    // );
    const signatureParams = getSignatureParameters(signature);

    console.log(
      (await context.polkadotApi.query.system.account(demoContract.options.address)).toHuman()
    );

    // const { result } = await context.createBlock(
    //   createTransaction(context, {
    //     ...BALTATHAR_TRANSACTION_TEMPLATE,
    //     to: demoContract.options.address,
    //     data: CALL_PERMIT_DEMO_INTERFACE.encodeFunctionData("bondFor", [
    //       ALITH_ADDRESS,
    //       42,
    //       "0x",
    //       100000,
    //       9999999999,
    //       signatureParams.v,
    //       signatureParams.r,
    //       signatureParams.s,
    //     ]),
    //     // value: Web3.utils.toWei("1", "ether"),
    //   })
    // );

    console.log(
      "ALITH",
      (await context.polkadotApi.query.system.account(ALITH_ADDRESS)).data.free.toHuman()
    );
    console.log(
      "CHARLETH",
      (await context.polkadotApi.query.system.account(CHARLETH_ADDRESS)).data.free.toHuman()
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: "0x000000000000000000000000000000000000080a",
        data: CALL_PERMIT_INTERFACE.encodeFunctionData("dispatch", [
          ALITH_ADDRESS,
          CHARLETH_ADDRESS,
          42,
          "0x",
          100000,
          9999999999,
          signatureParams.v,
          signatureParams.r,
          signatureParams.s,
        ]),
        // value: Web3.utils.toWei("1", "ether"),
      })
    );

    // result.events.forEach((e) => console.log(e.toHuman()));
    console.log(result.successful);

    console.log(
      (await context.polkadotApi.query.system.account(demoContract.options.address)).toHuman()
    );

    console.log(
      "ALITH",
      (await context.polkadotApi.query.system.account(ALITH_ADDRESS)).data.free.toHuman()
    );
    console.log(
      "CHARLETH",
      (await context.polkadotApi.query.system.account(CHARLETH_ADDRESS)).data.free.toHuman()
    );

    console.log(message);
    console.log(signature);
    console.log(signatureParams);

    const who = recoverTypedSignature({
      data: data as any,
      signature,
      version: SignTypedDataVersion.V4,
    });

    console.log(who);

    expectEVMResult(result.events, "Succeed");
  });
});

function getMessageData(message: any) {
  return {
    types: {
      EIP712Domain: [
        {
          name: "name",
          type: "string",
        },
        {
          name: "version",
          type: "string",
        },
        {
          name: "chainId",
          type: "uint256",
        },
        {
          name: "verifyingContract",
          type: "address",
        },
      ],
      CallPermit: [
        {
          name: "from",
          type: "address",
        },
        {
          name: "to",
          type: "address",
        },
        {
          name: "value",
          type: "uint256",
        },
        {
          name: "data",
          type: "bytes",
        },
        {
          name: "gaslimit",
          type: "uint64",
        },
        {
          name: "nonce",
          type: "uint256",
        },
        {
          name: "deadline",
          type: "uint256",
        },
      ],
    },
    primaryType: "CallPermit",
    domain: {
      name: "Call Permit Precompile",
      version: "1",
      chainId: 0,
      verifyingContract: "0x000000000000000000000000000000000000080a",
    },
    message: message,
  };
}
