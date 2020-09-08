import { expect } from "chai";

import { describeWithMoonbeam, customRequest, createAndFinalizeBlock } from "./util";
import { AbiItem } from "web3-utils";

describeWithMoonbeam("Moonbeam RPC (Gas)", `simple-specs.json`, (context) => {
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

	it("eth_estimateGas for contract creation", async function () {
		expect(
			await context.web3.eth.estimateGas({
				from: GENESIS_ACCOUNT,
				data: TEST_CONTRACT_BYTECODE,
			})
		).to.equal(91019);
	});

	it.skip("block gas limit over 5M", async function () {
		expect((await context.web3.eth.getBlock("latest")).gasLimit).to.be.above(5000000);
	});

	// Testing the gas limit protection, hardcoded to 25M
	it.skip("gas limit should decrease on next block if gas unused", async function () {
		this.timeout(15000);

		const gasLimit = (await context.web3.eth.getBlock("latest")).gasLimit;
		await createAndFinalizeBlock(context.web3);

		// Gas limit is expected to have decreased as the gasUsed by the block is lower than 2/3 of the previous gas limit
		const newGasLimit = (await context.web3.eth.getBlock("latest")).gasLimit;
		expect(newGasLimit).to.be.below(gasLimit);
	});

	// Testing the gas limit protection, hardcoded to 25M
	it.skip("gas limit should increase on next block if gas fully used", async function () {
		// TODO: fill a block with many heavy transaction to simulate lot of gas.
	});

	it("eth_estimateGas for contract call", async function () {
		const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
			from: GENESIS_ACCOUNT,
			gasPrice: "0",
		});

		expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
	});
});
