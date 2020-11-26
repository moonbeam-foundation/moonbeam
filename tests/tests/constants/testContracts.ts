import { AbiItem } from "web3-utils";

// Solidity: contract test {function multiply(uint a) public pure returns(uint d) {return a * 7;}}
export const TEST_CONTRACT_BYTECODE =
  "0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd" +
  "5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157" +
  "600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390" +
  "f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853" +
  "a30136fa6c5a7762af7364736f6c63430005110032";

export const TEST_CONTRACT_ABI = {
  constant: true,
  inputs: [{ internalType: "uint256", name: "a", type: "uint256" }],
  name: "multiply",
  outputs: [{ internalType: "uint256", name: "d", type: "uint256" }],
  payable: false,
  stateMutability: "pure",
  type: "function",
} as AbiItem;

export const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

// simple incremental count contract to test contract with state changes

// Solidity: 
// contract Test3 {
//   uint public count;

//   constructor() public {
//       count = 0;
//   }

//   function incr() public {
//       count=count+1;
//   }
// }
export const TEST_CONTRACT_BYTECODE_INCR =
  "6080604052348015600f57600080fd5b506000808190555060a5806100256000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c806306661abd146037578063119fbbd4146053575b600080fd5b603d605b565b6040518082815260200191505060405180910390f35b60596061565b005b60005481565b60016000540160008190555056fea26469706673582212204780263fff0edc01286caed1851cc629033bc25ec1f84995a71199017a4623dd64736f6c634300060b0033";

export const TEST_CONTRACT_INCR_ABI = [
	{
		"inputs": [],
		"stateMutability": "nonpayable",
		"type": "constructor"
	},
	{
		"inputs": [],
		"name": "count",
		"outputs": [
			{
				"internalType": "uint256",
				"name": "",
				"type": "uint256"
			}
		],
		"stateMutability": "view",
		"type": "function"
	},
	{
		"inputs": [],
		"name": "incr",
		"outputs": [],
		"stateMutability": "nonpayable",
		"type": "function"
	}
] as AbiItem[];

// infinite loop call

// Solidity: contract test {function infinite(uint a) public pure returns(uint d) {while (true) {}}}
export const INFINITE_CONTRACT_BYTECODE =
  "6080604052348015600f57600080fd5b5060788061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c80635bec9e6714602d575b600080fd5b60336035565b005b5b6001156040576036565b56fea264697066735822122015c7d339c1118112e1d9b33ea79ded52efa22f4e3cefe34097578a63e128f8a264736f6c63430007040033";

export const INFINITE_CONTRACT_ABI = {
  inputs: [],
  name: "infinite",
  outputs: [],
  stateMutability: "pure",
  type: "function",
} as AbiItem;

// infinite loop call with variable alocation

// Solidity: contract test {function infinite(uint a) public pure returns(uint d) {while (true) {data=data+1;}}}
export const INFINITE_CONTRACT_BYTECODE_VAR =
  "608060405234801561001057600080fd5b50600160008190555060b0806100276000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c80635bec9e6714603757806373d4a13a14603f575b600080fd5b603d605b565b005b60456074565b6040518082815260200191505060405180910390f35b5b600115607257600160005401600081905550605c565b565b6000548156fea264697066735822122053e7fd0d4629f7d9cd16b0456521ea0cf78e595e9627c45ee8a4f27f4119f39c64736f6c634300060b0033";

export const INFINITE_CONTRACT_ABI_VAR = [
  {
    inputs: [],
    stateMutability: "nonpayable",
    type: "constructor",
  },
  {
    inputs: [],
    name: "data",
    outputs: [
      {
        internalType: "uint256",
        name: "",
        type: "uint256",
      },
    ],
    stateMutability: "view",
    type: "function",
  },
  {
    inputs: [],
    name: "infinite",
    outputs: [],
    stateMutability: "nonpayable",
    type: "function",
  },
] as AbiItem[];


// definite loop call with variable alocation

// Solidity:
// contract Test4 {
//     uint public count;

//     constructor() public {
//         count = 0;
//     }

//     function incr(uint n) public {
//         uint i=0;
//         while (i<n) {
//             count=count+1;
//             i+=1;
//         }
//     }
// }

export const FINITE_LOOP_CONTRACT_BYTECODE =
  "608060405234801561001057600080fd5b506000808190555060e1806100266000396000f3fe6080604052348015600f57600080fd5b506004361060325760003560e01c806306661abd14603757806321b13c48146053575b600080fd5b603d607e565b6040518082815260200191505060405180910390f35b607c60048036036020811015606757600080fd5b81019080803590602001909291905050506084565b005b60005481565b60008090505b8181101560a757600160005401600081905550600181019050608a565b505056fea264697066735822122055c3057e9a4de212a72858fab41c167c7c616b47ec2ce4e7e1ebf152e8f83dc464736f6c634300060b0033";

export const FINITE_LOOP_CONTRACT_ABI= [
	{
		"inputs": [],
		"stateMutability": "nonpayable",
		"type": "constructor"
	},
	{
		"inputs": [],
		"name": "count",
		"outputs": [
			{
				"internalType": "uint256",
				"name": "",
				"type": "uint256"
			}
		],
		"stateMutability": "view",
		"type": "function"
	},
	{
		"inputs": [
			{
				"internalType": "uint256",
				"name": "n",
				"type": "uint256"
			}
		],
		"name": "incr",
		"outputs": [],
		"stateMutability": "nonpayable",
		"type": "function"
	}
] as AbiItem[];