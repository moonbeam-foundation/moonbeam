import { expect } from "chai";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY, TEST_CONTRACT_BYTECODE } from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

describeWithMoonbeam("Frontier RPC (Pending Pool)", `simple-specs.json`, (context) => {
  // Solidity: contract test { function multiply(uint a) public pure returns(uint d)
  // {return a * 7;}}

  /*
    At rpc-level, there is no interface for retrieving emulated pending transactions - emulated
    transactions that exist in the Substrate's pending transaction pool. Instead they are added to a
    shared collection (Mutex) with get/set locking to serve requests that ask for this transactions
    information before they are included in a block.

    We want to test that:
      - We resolve multiple promises in parallel that will write in this collection on the rpc-side
      - We resolve multiple promises in parallel that will read from this collection on the rpc-side
      - We can get the final transaction data once it leaves the pending collection
  */
  it("should handle pending transactions", async function () {
    const NBR_TXNS = 10;
    let promises = [];
    let tx_hashes = [];
    let responses;

    let i;
    let nonce = 0;
    for (i = 0; i < NBR_TXNS; i++) {
      let txn = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: TEST_CONTRACT_BYTECODE,
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
          nonce: nonce,
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      promises.push(customRequest(context.web3, "eth_sendRawTransaction", [txn.rawTransaction]));
      nonce += 1;
    }

    await Promise.all(promises).then((responses) => {
      promises = [];
      for (let r of responses) {
        tx_hashes.push(r.result);
        promises.push(customRequest(context.web3, "eth_getTransactionByHash", [r.result]));
      }
    });
    // Expect a unique set of transaction hashes.
    expect(tx_hashes.length).to.be.eq(new Set(tx_hashes).size);

    await Promise.all(promises).then((responses) => {
      // Expect a response for each transaction hash.
      expect(responses.length).to.be.eq(tx_hashes.length);
      // Expect each transaction hash to have a response.
      expect([...new Set(tx_hashes)].sort()).deep.eq(
        responses
          .map(function (a) {
            return a.result.hash;
          })
          .sort()
      );
      for (i = 0; i < tx_hashes.length; i++) {
        let tx_data = responses[i].result;
        // Expect the transaction to not be aware of it's block.
        expect(tx_data.blockNumber).to.be.null;
        // Expect the transaction to not be aware of it's index.
        expect(tx_data.transactionIndex).to.be.null;
      }
    });

    await createAndFinalizeBlock(context.polkadotApi);

    await Promise.all(promises).then((responses) => {
      promises = [];
      for (let tx_hash of tx_hashes) {
        promises.push(customRequest(context.web3, "eth_getTransactionByHash", [tx_hash]));
      }
    });

    await Promise.all(promises).then((responses) => {
      for (let r of responses) {
        // Expect the transaction to be aware of it's block.
        expect(r.result.blockNumber).to.not.be.null;
        // Expect the transaction to be aware of it's index.
        expect(r.result.transactionIndex).to.not.be.null;
      }
      // Expect each transaction to have a unique index.
      expect(
        [
          ...new Set(
            responses.map(function (a) {
              return a.result.transactionIndex;
            })
          ),
        ].sort()
      ).deep.eq(
        responses
          .map(function (a) {
            return a.result.transactionIndex;
          })
          .sort()
      );
    });
  });
});
