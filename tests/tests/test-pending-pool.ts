import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithFrontier } from "./util";

describeWithFrontier("Frontier RPC (Pending Pool)", `simple-specs.json`, (context) => {
	const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

	// Solidity: contract test { function multiply(uint a) public pure returns(uint d) {return a * 7;}}
	const TEST_CONTRACT_BYTECODE =
		"0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005110032";
	const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

	it("should return a pending transaction", async function () {
		this.timeout(15000);
		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: TEST_CONTRACT_BYTECODE,
				value: "0x00",
				gasPrice: "0x01",
				gas: "0x100000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);

		const tx_hash = (await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])).result;

		const pending_transaction = (await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])).result;
		// pending transactions do not know yet to which block they belong to
		expect(pending_transaction).to.include({
			blockNumber: null,
			hash: tx_hash,
			publicKey: "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529fa2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
			r: "0x5431b25e8100a21ced6af01868357b19d58b94afa6f57dc7cbf81f4a922ddecc",
			s: "0x22e05530d015ea702ffb37af313e691ce4423565c2734c267edd3c74aea0a010",
			v: "0x77",
		});

		await createAndFinalizeBlock(context.web3);

		const processed_transaction = (await customRequest(context.web3, "eth_getTransactionByHash", [tx_hash])).result;
		expect(processed_transaction).to.include({
			blockNumber: "0x1",
			hash: tx_hash,
			publicKey: "0x624f720eae676a04111631c9ca338c11d0f5a80ee42210c6be72983ceb620fbf645a96f951529fa2d70750432d11b7caba5270c4d677255be90b3871c8c58069",
			r: "0x5431b25e8100a21ced6af01868357b19d58b94afa6f57dc7cbf81f4a922ddecc",
			s: "0x22e05530d015ea702ffb37af313e691ce4423565c2734c267edd3c74aea0a010",
			v: "0x77",
		});
	});
});
