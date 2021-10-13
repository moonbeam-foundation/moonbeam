import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import { ethers } from "ethers";

import {
  GENESIS_ACCOUNT,
  ALITH,
  BALTATHAR,
  ALITH_PRIV_KEY,
  CHARLETH,
  BALTATHAR_PRIV_KEY,
} from "../util/constants";
import { createTransaction } from "../util/transactions";

const ADDRESS_XTOKENS = "0x0000000000000000000000000000000000000804";
const BALANCES_ADDRESS = "0x0000000000000000000000000000000000000802";

var contractAbi = [
	{
		"inputs": [
			{
				"internalType": "address",
				"name": "currency_address",
				"type": "address"
			},
			{
				"internalType": "uint256",
				"name": "amount",
				"type": "uint256"
			},
			{
				"components": [
					{
						"internalType": "uint8",
						"name": "parents",
						"type": "uint8"
					},
					{
						"internalType": "bytes[]",
						"name": "interior",
						"type": "bytes[]"
					}
				],
				"internalType": "struct Xtokens.Multilocation",
				"name": "destination",
				"type": "tuple"
			},
			{
				"internalType": "uint64",
				"name": "weight",
				"type": "uint64"
			}
		],
		"name": "transfer",
		"outputs": [],
		"stateMutability": "nonpayable",
		"type": "function"
	},
	{
		"inputs": [
			{
				"components": [
					{
						"internalType": "uint8",
						"name": "parents",
						"type": "uint8"
					},
					{
						"internalType": "bytes[]",
						"name": "interior",
						"type": "bytes[]"
					}
				],
				"internalType": "struct Xtokens.Multilocation",
				"name": "asset",
				"type": "tuple"
			},
			{
				"internalType": "uint256",
				"name": "amount",
				"type": "uint256"
			},
			{
				"components": [
					{
						"internalType": "uint8",
						"name": "parents",
						"type": "uint8"
					},
					{
						"internalType": "bytes[]",
						"name": "interior",
						"type": "bytes[]"
					}
				],
				"internalType": "struct Xtokens.Multilocation",
				"name": "destination",
				"type": "tuple"
			},
			{
				"internalType": "uint64",
				"name": "weight",
				"type": "uint64"
			}
		],
		"name": "transfer_multiasset",
		"outputs": [],
		"stateMutability": "nonpayable",
		"type": "function"
	}
]
const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

describeDevMoonbeam("Precompiles - xtokens", (context) => {
  let iFace;
  before("Deploy contract", async () => {
    iFace = new ethers.utils.Interface(contractAbi)
  });

  it("allows to issue transfer xtokens", async function () {
    const data = iFace.encodeFunctionData("transfer",[BALANCES_ADDRESS, 100, [1, ["0x01010101010101010101010101010101010101010101010101010101010101010100"]], 100]);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_XTOKENS,
        data: data,
      },
    ]);
	// result only exists if the call is succesful
    expect(tx_call.hasOwnProperty('result'));
  });
});