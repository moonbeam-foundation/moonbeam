import { expect } from "chai";
import { Keyring } from "@polkadot/keyring";
import { step } from "mocha-steps";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

const CONTRACT = require("./constants/Incrementer.json");


const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

describeWithMoonbeam("Moonbeam RPC (Trace)", `simple-specs.json`, (context) => {
    step("should replay over an intermediate state", async function () {

        const createTx = await context.web3.eth.accounts.signTransaction(
            {
                from: GENESIS_ACCOUNT,
                data: CONTRACT.bytecode,
                value: "0x00",
                gasPrice: "0x01",
                gas: "0x100000",
            },
            GENESIS_ACCOUNT_PRIVATE_KEY
        );
        let send = await customRequest(
            context.web3, "eth_sendRawTransaction", [createTx.rawTransaction]
        );
        await createAndFinalizeBlock(context.polkadotApi);
        let receipt = await context.web3.eth.getTransactionReceipt(send.result);
        // This contract's `sum` method receives a number as an argument, increments the storage and
        // returns the current value.
        let contract = new context.web3.eth.Contract(CONTRACT.abi, receipt.contractAddress);

        // In our case, the total number of transactions == the max value of the incrementer.
        // If we trace the last transaction of the block, should return the total number of
        // transactions we executed (10).
        // If we trace the 5th transaction, should return 5 and so on.
        //
        // So we set 5 different targets for each block: the 1st, 3 intermediate, and the last.
        const total_txs = 10;
        let targets = [1, 2, 5, 8, 10];
        let iteration = 0;
        for (let target of targets) {
            let txs = [];
            let num_txs;
            for (num_txs = 1; num_txs <= total_txs; num_txs++) {
                let callTx = await context.web3.eth.accounts.signTransaction({
                    from: GENESIS_ACCOUNT,
                    to: receipt.contractAddress,
                    gas: "0x100000",
                    value: "0x00",
                    nonce: num_txs + (iteration * total_txs),
                    data: contract.methods.sum(1).encodeABI() // increments by one
                }, GENESIS_ACCOUNT_PRIVATE_KEY);

                send = await customRequest(
                    context.web3, "eth_sendRawTransaction", [callTx.rawTransaction]
                );
                txs.push(send.result);
            }
            await createAndFinalizeBlock(context.polkadotApi);

            let index = target - 1;
            let intermediate_tx = await customRequest(
                context.web3, "debug_traceTransaction", [txs[index]]
            );

            let evm_result = context.web3.utils.hexToNumber(
                context.web3.utils.bytesToHex(
                    intermediate_tx.result.returnValue
                )
            );
            let expected = target + (iteration * total_txs);
            console.log(`Matching target ${expected} against evm result ${evm_result}`);
            expect(evm_result).to.equal(expected);
            iteration += 1;
        }
    });
});