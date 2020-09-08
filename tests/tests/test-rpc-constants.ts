import { expect } from "chai";
import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

// All test for the RPC

describeWithMoonbeam("Moonbeam RPC (Constant)", `simple-specs.json`, (context) => {
	it("should have 0 hashrate", async function () {
		expect(await context.web3.eth.getHashrate()).to.equal(0);
	});

	it("should have chainId 42", async function () {
		// The chainId for moonbeam is 43
		expect(await context.web3.eth.getChainId()).to.equal(43);
	});

	it("should have no account", async function () {
		expect(await context.web3.eth.getAccounts()).to.eql([]);
	});

	it("block author should be 0x0000000000000000000000000000000000000000", async function () {
		// This address `0x1234567890` is hardcoded into the runtime find_author
		// as we are running manual sealing consensus.
		expect(await context.web3.eth.getCoinbase()).to.equal("0x0000000000000000000000000000000000000000");
	});
});
