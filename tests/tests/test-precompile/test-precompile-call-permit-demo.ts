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
  PRECOMPILE_CALL_PERMIT_ADDRESS,
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
import { web3EthCall } from "../../util/providers";

const CALL_PERMIT_DEMO_CONTRACT_JSON = getCompiled("CallPermitDemo");
const CALL_PERMIT_DEMO_INTERFACE = new ethers.utils.Interface(
  CALL_PERMIT_DEMO_CONTRACT_JSON.contract.abi
);

const CALL_PERMIT_CONTRACT_JSON = getCompiled("precompiles/call-permit/CallPermit");
const CALL_PERMIT_INTERFACE = new ethers.utils.Interface(CALL_PERMIT_CONTRACT_JSON.contract.abi);

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
  before(
    "setup contract and bond for baltathar, and for alith via baltathar using permit",
    async function () {
      const { contract, rawTx } = await createContract(context, "CallPermitDemo", {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 5_000_000,
      });
      await context.createBlock(rawTx);
      demoContract = contract;

      const { result: bondAmountResult } = await web3EthCall(context.web3, {
        to: demoContract.options.address,
        data: CALL_PERMIT_DEMO_INTERFACE.encodeFunctionData("BOND_AMOUNT"),
      });
      const bondAmount = Number(bondAmountResult);

      // bond baltathar
      const { result: baltatharResult } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: demoContract.options.address,
          data: CALL_PERMIT_DEMO_INTERFACE.encodeFunctionData("bond"),
          gas: 200_000,
          value: bondAmount,
        })
      );
      expectEVMResult(baltatharResult.events, "Succeed");

      // bond alice via baltathar using call permit
      const { result: alithNonceResult } = await web3EthCall(context.web3, {
        to: PRECOMPILE_CALL_PERMIT_ADDRESS,
        data: CALL_PERMIT_INTERFACE.encodeFunctionData("nonces", [ALITH_ADDRESS]),
      });
      const alithNonce = Number(alithNonceResult);
      const signature = signTypedData({
        privateKey: Buffer.from(ALITH_PRIVATE_KEY.substring(2), "hex"),
        data: {
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
            chainId: 1281,
            verifyingContract: PRECOMPILE_CALL_PERMIT_ADDRESS,
          },
          message: {
            from: ALITH_ADDRESS,
            to: demoContract.options.address,
            value: bondAmount,
            data: "",
            gaslimit: 100_000,
            nonce: alithNonce,
            deadline: 9999999999,
          },
        },
        version: SignTypedDataVersion.V4,
      });
      const { v, r, s } = getSignatureParameters(signature);

      const { result: baltatharForAlithResult } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: demoContract.options.address,
          data: CALL_PERMIT_DEMO_INTERFACE.encodeFunctionData("bondFor", [
            ALITH_ADDRESS,
            100_000,
            9999999999,
            v,
            r,
            s,
          ]),
          gas: 200_000,
        })
      );
      expectEVMResult(baltatharForAlithResult.events, "Succeed");
    }
  );

  it("should have bonds for baltathar and alith in contract balance", async function () {
    const freeBalance = (
      await context.polkadotApi.query.system.account(demoContract.options.address)
    ).data.free.toNumber();
    expect(freeBalance).to.equal(200);
  });

  it("should have bond for baltathar in contract storage", async function () {
    const { result: baltatharBond } = await web3EthCall(context.web3, {
      to: demoContract.options.address,
      data: CALL_PERMIT_DEMO_INTERFACE.encodeFunctionData("getBondAmount", [BALTATHAR_ADDRESS]),
    });
    expect(Number(baltatharBond)).to.equal(100);
  });

  it("should have bond for alith in contract storage", async function () {
    const { result: alithBond } = await web3EthCall(context.web3, {
      to: demoContract.options.address,
      data: CALL_PERMIT_DEMO_INTERFACE.encodeFunctionData("getBondAmount", [ALITH_ADDRESS]),
    });
    expect(Number(alithBond)).to.equal(100);
  });
});
