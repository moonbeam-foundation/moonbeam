import { expect } from "chai";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

const CONTRACT = require("./constants/TraceFilter.json");

const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GENESIS_ACCOUNT_PRIVATE_KEY =
	"0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";


const address0 = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";
const address1 = "0x42e2ee7ba8975c473157634ac2af4098190fc741";
const address2 = "0xf8cef78e923919054037a1d03662bbd884ff4edf";

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
		expect(response.result[0].action.input).to.equal("0x608060405234801561001057600080fd5b506040516103783803806103788339818101604052602081101561003357600080fd5b8101908080519060200190929190505050801561004f57600080fd5b506103198061005f6000396000f3fe608060405234801561001057600080fd5b506004361061004c5760003560e01c80635eaf9bc114610051578063a885f4e31461005b578063cb30e6961461009f578063f34f1610146100a9575b600080fd5b61005961010d565b005b61009d6004803603602081101561007157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061010f565b005b6100a76101d2565b005b61010b600480360360408110156100bf57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506101d7565b005b565b8073ffffffffffffffffffffffffffffffffffffffff16635eaf9bc16040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561015757600080fd5b505af115801561016b573d6000803e3d6000fd5b505050508073ffffffffffffffffffffffffffffffffffffffff1663cb30e6966040518163ffffffff1660e01b8152600401600060405180830381600087803b1580156101b757600080fd5b505af11580156101cb573d6000803e3d6000fd5b5050505050565b600080fd5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b15801561024057600080fd5b505af1925050508015610251575060015b61025a5761025b565b5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b1580156102c457600080fd5b505af19250505080156102d5575060015b6102de576102df565b5b505056fea2646970667358221220ee197ab2a083888733b5308bb086dad0368108fe41328cd7752f955e7e67232464736f6c634300070600330000000000000000000000000000000000000000000000000000000000000000");
		expect(response.result[0].action.value).to.equal("0x0");
		expect(response.result[0].blockHash).to.equal("0xdd9d02101561bf442989f06cafdb1b8c4db61b7edf8984340422aa22758a5050");
		expect(response.result[0].blockNumber).to.equal(1);
		expect(response.result[0].result.address).to.equal("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a");
		expect(response.result[0].result.code).to.equal("0x608060405234801561001057600080fd5b506004361061004c5760003560e01c80635eaf9bc114610051578063a885f4e31461005b578063cb30e6961461009f578063f34f1610146100a9575b600080fd5b61005961010d565b005b61009d6004803603602081101561007157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061010f565b005b6100a76101d2565b005b61010b600480360360408110156100bf57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506101d7565b005b565b8073ffffffffffffffffffffffffffffffffffffffff16635eaf9bc16040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561015757600080fd5b505af115801561016b573d6000803e3d6000fd5b505050508073ffffffffffffffffffffffffffffffffffffffff1663cb30e6966040518163ffffffff1660e01b8152600401600060405180830381600087803b1580156101b757600080fd5b505af11580156101cb573d6000803e3d6000fd5b5050505050565b600080fd5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b15801561024057600080fd5b505af1925050508015610251575060015b61025a5761025b565b5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b1580156102c457600080fd5b505af19250505080156102d5575060015b6102de576102df565b5b505056fea2646970667358221220ee197ab2a083888733b5308bb086dad0368108fe41328cd7752f955e7e67232464736f6c63430007060033");
		expect(response.result[0].result.gasUsed).to.equal("0x153");
		expect(response.result[0].error).to.equal(undefined);
		expect(response.result[0].subtraces).to.equal(0);
		expect(response.result[0].traceAddress.length).to.equal(0);
		expect(response.result[0].transactionHash).to.equal("0x282fdd0b08fd385bbc233cffd5679ee703fc6b4c5b54d6096ae47fa92372790e");
		expect(response.result[0].transactionPosition).to.equal(0);
		expect(response.result[0].type).to.equal("create");
	})

	step("Replay reverting CREATE", async function () {
		// Deploy contract
		const contract = new context.web3.eth.Contract(CONTRACT.abi);
		const contract_deploy = contract.deploy({
			data: CONTRACT.bytecode,
			arguments: [true] // revert
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

		// Perform RPC call.
		let response = await customRequest(context.web3, "trace_filter", [
			{
				fromBlock: "0x02",
				toBlock: "0x02",
			}
		]);

		// console.log(JSON.stringify(response));

		expect(response.result.length).to.equal(1);
		expect(response.result[0].action.createMethod).to.equal("create");
		expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
		expect(response.result[0].action.gas).to.equal("0x4fff44");
		expect(response.result[0].action.input).to.equal("0x608060405234801561001057600080fd5b506040516103783803806103788339818101604052602081101561003357600080fd5b8101908080519060200190929190505050801561004f57600080fd5b506103198061005f6000396000f3fe608060405234801561001057600080fd5b506004361061004c5760003560e01c80635eaf9bc114610051578063a885f4e31461005b578063cb30e6961461009f578063f34f1610146100a9575b600080fd5b61005961010d565b005b61009d6004803603602081101561007157600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16906020019092919050505061010f565b005b6100a76101d2565b005b61010b600480360360408110156100bf57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001909291905050506101d7565b005b565b8073ffffffffffffffffffffffffffffffffffffffff16635eaf9bc16040518163ffffffff1660e01b8152600401600060405180830381600087803b15801561015757600080fd5b505af115801561016b573d6000803e3d6000fd5b505050508073ffffffffffffffffffffffffffffffffffffffff1663cb30e6966040518163ffffffff1660e01b8152600401600060405180830381600087803b1580156101b757600080fd5b505af11580156101cb573d6000803e3d6000fd5b5050505050565b600080fd5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b15801561024057600080fd5b505af1925050508015610251575060015b61025a5761025b565b5b8173ffffffffffffffffffffffffffffffffffffffff1663a885f4e3826040518263ffffffff1660e01b8152600401808273ffffffffffffffffffffffffffffffffffffffff168152602001915050600060405180830381600087803b1580156102c457600080fd5b505af19250505080156102d5575060015b6102de576102df565b5b505056fea2646970667358221220ee197ab2a083888733b5308bb086dad0368108fe41328cd7752f955e7e67232464736f6c634300070600330000000000000000000000000000000000000000000000000000000000000001");
		expect(response.result[0].action.value).to.equal("0x0");
		expect(response.result[0].blockHash).to.equal("0x609024beeb421348533180334dbc3bb74549ce1f2de55921230f8df0bf1c869f");
		expect(response.result[0].blockNumber).to.equal(2);
		expect(response.result[0].result).to.equal(undefined);
		expect(response.result[0].error).to.equal("Reverted");
		expect(response.result[0].subtraces).to.equal(0);
		expect(response.result[0].traceAddress.length).to.equal(0);
		expect(response.result[0].transactionHash).to.equal("0x214cf6578d15751c7d5e68ad7167f2b7bcbb0023be155cd55cd1fb059e238c89");
		expect(response.result[0].transactionPosition).to.equal(0);
		expect(response.result[0].type).to.equal("create");
	})

	step("Multiple transactions in the same block + trace over multiple blocks", async function () {
		const contract = new context.web3.eth.Contract(CONTRACT.abi);

		// Deploy 2 more contracts
		for (var i = 0; i < 2; i++) {
			const contract_deploy = contract.deploy({
				data: CONTRACT.bytecode,
				arguments: [false] // don't revert
			});

			const tx = await context.web3.eth.accounts.signTransaction(
				{
					nonce: 2+i,
					from: GENESIS_ACCOUNT,
					data: contract_deploy.encodeABI(),
					value: "0x00",
					gasPrice: "0x01",
					gas: "0x100000",
				},
				GENESIS_ACCOUNT_PRIVATE_KEY
			);

			let send = await customRequest(context.web3, "eth_sendRawTransaction", [
				tx.rawTransaction,
			]);			
		}

		await createAndFinalizeBlock(context.polkadotApi);

		// Perform RPC call.
		let response = await customRequest(context.web3, "trace_filter", [
			{
				fromBlock: "0x02",
				toBlock: "0x03",
			}
		]);

		expect(response.result.length).to.equal(3);
		expect(response.result[0].blockNumber).to.equal(2);
		expect(response.result[0].transactionPosition).to.equal(0);
		expect(response.result[1].blockNumber).to.equal(3);
		expect(response.result[1].transactionPosition).to.equal(0);
		expect(response.result[2].blockNumber).to.equal(3);
		expect(response.result[2].transactionPosition).to.equal(1);
		
		// console.log(JSON.stringify(response));
	})

	step("Call with subcalls, some reverting", async function () {
		const contract = new context.web3.eth.Contract(CONTRACT.abi);

		const contract_call = contract.methods.subcalls(address1, address2);

		const tx = await context.web3.eth.accounts.signTransaction(
			{
				to: address0,
				from: GENESIS_ACCOUNT,
				data: contract_call.encodeABI(),
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

		// Perform RPC call.
		let response = await customRequest(context.web3, "trace_filter", [
			{
				fromBlock: "0x04",
				toBlock: "0x04",
			}
		]);

		// console.log(JSON.stringify(response));
		expect(response.result.length).to.equal(7);
		expect(response.result[0].subtraces).to.equal(2);
		expect(response.result[0].traceAddress).to.deep.equal([]);
		expect(response.result[1].subtraces).to.equal(2);
		expect(response.result[1].traceAddress).to.deep.equal([0]);
		expect(response.result[2].subtraces).to.equal(0);
		expect(response.result[2].traceAddress).to.deep.equal([0,0]);
		expect(response.result[3].subtraces).to.equal(0);
		expect(response.result[3].traceAddress).to.deep.equal([0,1]);
		expect(response.result[4].subtraces).to.equal(2);
		expect(response.result[4].traceAddress).to.deep.equal([1]);
		expect(response.result[5].subtraces).to.equal(0);
		expect(response.result[5].traceAddress).to.deep.equal([1,0]);
		expect(response.result[6].subtraces).to.equal(0);
		expect(response.result[6].traceAddress).to.deep.equal([1,1]);
	})
})