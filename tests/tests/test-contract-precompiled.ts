import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { AbiItem } from "web3-utils";

describeWithMoonbeam("Moonbeam RPC (Contract of precompiled function)", `simple-specs.json`, (context) => {
	const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
	const GENESIS_ACCOUNT_PRIVATE_KEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

    const TEST_CONTRACT_ABI = {
        constant: false,
        // keep in mind that `testit(bytes8)` is the signature given here,
        // this will affect the data that will be transmitted to the function
        // in this case, assuming the data is 0xdeadbeef, we calculate (using pyethereum):
        // >>> ethereum.utils.sha3("testit(bytes8)").hex()
        // which will result in 0xba9cf00f603084017b95e68b576cac531a2fcba78b2b0773614428d7905c09ff
        // and the first 4 bytes of that (the function signature) will be part of that
        // while it's obvious this is how ethereum selects functions, it's not obvious
        // that with precompiles, YOU can alter this using this ABI
        inputs: [{ name: "a", type: "bytes" }],
        name: "testit",
        outputs: [{ name: "res", type: "bytes4" }],
        payable: false,
        stateMutability: "pure",
        type: "function",
    } as AbiItem;

    const FIRST_CONTRACT_ADDRESS = "0000000000000000000000000000000000001000";

    before("create the contract", async function () {
        await createAndFinalizeBlock(context.web3);
    });

    it("should return contract method result", async function () {
        const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
            from: GENESIS_ACCOUNT,
            gasPrice: "0x01",
            gas: 10000000
        });

        // the function is expected to return deadbeef regardless of what you give it
        expect(await contract.methods.testit('0x12345678').call()).to.equal('0xdeadbeef');
    });
});
