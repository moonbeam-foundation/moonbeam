import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

const CONTRACT = require("./constants/TraceFilter.json");

const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GENESIS_ACCOUNT_PRIVATE_KEY =
	"0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

describeWithMoonbeam("Moonbeam RPC (trace_filter)", `simple-specs.json`, (context) => {
	step("Replay succeeding CREATE", async function () {
		// Deploy contract
		const contract = new context.web3.eth.Contract(CONTRACT.abi);
		const contract_deploy = contract.deploy({
			data: CONTRACT.bytecode,
			arguments: [false] // don't revert
		});

		const tx = await context.web3.eth.accounts.signTransaction(
			{
				from: GENESIS_ACCOUNT,
				data: contract_deploy.encodeABI(),
				value: "0x00",
				gasPrice: "0x01",
				gas: "0x500000",
			},
			GENESIS_ACCOUNT_PRIVATE_KEY
		);

		let send = await customRequest(context.web3, "eth_sendRawTransaction", [
			tx.rawTransaction,
		]);

		await createAndFinalizeBlock(context.polkadotApi);
		let receipt = await context.web3.eth.getTransactionReceipt(send.result);

		// Perform RPC call.
		let response = await customRequest(context.web3, "trace_filter", [
			{
				fromBlock: "0x01",
				toBlock: "0x01",
			}
		]);

		// console.log(JSON.stringify(response));

		expect(response.result.length).to.equal(1);
		expect(response.result[0].action.createMethod).to.equal("create");
		expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
		expect(response.result[0].action.gas).to.equal("0x4ffead");
		expect(response.result[0].action.input).to.equal("0x608060405234801561001057600080fd5b506040516103723803806103728339818101604052602081101561003357600080fd5b8101908080519060200190929190505050801561004f57600080fd5b506103138061005f6000396000f3fe608060405234801561001057600080fd5b506004361061004c5760003560e01c80635eaf9bc114610051578063a885f4e31461005b578063cb30e6961461009f578063f34f1610146100a9575b600080fd5b61005961010d565b005b61009d6004803603602081101561007157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061010f565b005b6100a76101d2565b005b61010b600480360360408110156100bf57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506101d7565b005b565b8073ffffffffffffffffffffffffffffffffffffffff16635eaf9bc16040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561015757600080fd5b505af115801561016b573d6000803e3d6000fd5b505050508073ffffffffffffffffffffffffffffffffffffffff1663cb30e6966040518163ffffffff1660e01b8152600401600060405180830381600087803b1580156101b757600080fd5b505af11580156101cb573d6000803e3d6000fd5b5050505050565b600080fd5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b15801561024057600080fd5b505af1158015610254573d6000803e3d6000fd5b505050508173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b1580156102c157600080fd5b505af11580156102d5573d6000803e3d6000fd5b50505050505056fea2646970667358221220ce4909dd4038cf1f27b31d3b55f1c31756d32e5dbc00afe106f318e3e7e7e45f64736f6c634300070400330000000000000000000000000000000000000000000000000000000000000000");
		expect(response.result[0].action.value).to.equal("0x0");
		expect(response.result[0].blockHash).to.equal("0x4b518bd6cf72f4605e4568babed6459b08512c49dffce8c7848235848f557189");
		expect(response.result[0].blockNumber).to.equal(1);
		expect(response.result[0].result.address).to.equal("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a");
		expect(response.result[0].result.code).to.equal("0x608060405234801561001057600080fd5b506004361061004c5760003560e01c80635eaf9bc114610051578063a885f4e31461005b578063cb30e6961461009f578063f34f1610146100a9575b600080fd5b61005961010d565b005b61009d6004803603602081101561007157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061010f565b005b6100a76101d2565b005b61010b600480360360408110156100bf57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506101d7565b005b565b8073ffffffffffffffffffffffffffffffffffffffff16635eaf9bc16040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561015757600080fd5b505af115801561016b573d6000803e3d6000fd5b505050508073ffffffffffffffffffffffffffffffffffffffff1663cb30e6966040518163ffffffff1660e01b8152600401600060405180830381600087803b1580156101b757600080fd5b505af11580156101cb573d6000803e3d6000fd5b5050505050565b600080fd5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b15801561024057600080fd5b505af1158015610254573d6000803e3d6000fd5b505050508173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b1580156102c157600080fd5b505af11580156102d5573d6000803e3d6000fd5b50505050505056fea2646970667358221220ce4909dd4038cf1f27b31d3b55f1c31756d32e5dbc00afe106f318e3e7e7e45f64736f6c63430007040033");
		expect(response.result[0].result.gasUsed).to.equal("0x153");
		expect(response.result[0].error).to.equal(undefined);
		expect(response.result[0].subtraces).to.equal(0);
		expect(response.result[0].traceAddress.length).to.equal(0);
		expect(response.result[0].transactionHash).to.equal("0xa361f30f4abb2c060594c5f4ddd0b475bc993b482d622745a3ce1c5a71b564cc");
		expect(response.result[0].transactionPosition).to.equal(0);
		expect(response.result[0].type).to.equal("create");
	})
})