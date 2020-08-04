import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { AbiItem } from "web3-utils";

describeWithMoonbeam("Moonbeam RPC (Contract Methods)", `simple-specs.json`, (context) => {
	const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

	// Solidity: contract test { function multiply(uint a) public pure returns(uint d) {return a * 7;}}
	const TEST_CONTRACT_BYTECODE =
		"0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005110032";

	const TEST_CONTRACT_ABI = {
		constant: true,
		inputs: [{ internalType: "uint256", name: "a", type: "uint256" }],
		name: "multiply",
		outputs: [{ internalType: "uint256", name: "d", type: "uint256" }],
		payable: false,
		stateMutability: "pure",
		type: "function",
	} as AbiItem;

	const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a"; // Those test are ordered. In general this should be avoided, but due to the time it takes	// to spin up a Moonbeam node, it saves a lot of time.

	before("create the contract", async function () {
		this.timeout(15000);
		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: TEST_CONTRACT_BYTECODE,
				value: "0x00",
				gasPrice: "0x00",
				gas: "0x100000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);
		await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
		await createAndFinalizeBlock(context.web3);
	});

	it("should return contract method result", async function () {
		const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0",
		});

		expect(await contract.methods.multiply(3).call()).to.equal("21");
	});

	// Requires error handling
	it.skip("should fail for missing parameters", async function () {
		const contract = new context.web3.eth.Contract([{ ...TEST_CONTRACT_ABI, inputs: [] }], FIRST_CONTRACT_ADDRESS, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0",
		});
		await contract.methods
			.multiply()
			.call()
			.catch((err) =>
				expect(err.message).to.equal(`Returned error: VM Exception while processing transaction: revert.`)
			);
	});

	// Requires error handling
	it.skip("should fail for too many parameters", async function () {
		const contract = new context.web3.eth.Contract(
			[
				{
					...TEST_CONTRACT_ABI,
					inputs: [
						{ internalType: "uint256", name: "a", type: "uint256" },
						{ internalType: "uint256", name: "b", type: "uint256" },
					],
				},
			],
			FIRST_CONTRACT_ADDRESS,
			{
				from: GENESIS_ACCOUNT,
				gasPrice: "0",
			}
		);
		await contract.methods
			.multiply(3, 4)
			.call()
			.catch((err) =>
				expect(err.message).to.equal(`Returned error: VM Exception while processing transaction: revert.`)
			);
	});

	// Requires error handling
	it.skip("should fail for invalid parameters", async function () {
		const contract = new context.web3.eth.Contract(
			[{ ...TEST_CONTRACT_ABI, inputs: [{ internalType: "address", name: "a", type: "address" }] }],
			FIRST_CONTRACT_ADDRESS,
			{ from: GENESIS_ACCOUNT, gasPrice: "0" }
		);
		await contract.methods
			.multiply("0x0123456789012345678901234567890123456789")
			.call()
			.catch((err) =>
				expect(err.message).to.equal(`Returned error: VM Exception while processing transaction: revert.`)
			);
	});
});
