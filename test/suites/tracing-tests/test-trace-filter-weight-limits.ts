import { afterAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { sleep } from "../../helpers";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "T18",
  title: "Trace filter - Test block weight limits with evm-tracing enabled",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    afterAll(async () => {
      await sleep(500); // Add sleep to allow for graceful teardown
    });

    it({
      id: "T01",
      title:
        "The number of unique transaction traces should be the same as the number of transactions included in a block",
      test: async function () {
        const { abi, contractAddress } = await context.deployContract!("BloatedContract");

        let nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });
        const tx = [];
        for (let i = 0; i < 500; i++) {
          tx.push(
            await createEthersTransaction(context, {
              to: contractAddress,
              data: encodeFunctionData({ abi, functionName: "doSomething", args: [] }),
              gasLimit: 10_000_000,
              nonce: nonce++,
            })
          );
        }

        const substrateBlock = await context.createBlock(tx, { allowFailures: false });
        const txHashes = (substrateBlock.result || [])
          .filter((result) => result.successful)
          .map((result) => result.hash);

        const blockNumber = (await context.polkadotJs().query.ethereum.currentBlock())
          .unwrap()
          .header.number.toNumber();
        const blockNumberHex = blockNumber.toString(16);
        const ethBlock = await customDevRpcRequest("eth_getBlockByNumber", [blockNumberHex, false]);

        // Confirm that all transactions were included in the ethereum block
        expect(ethBlock.transactions.length).to.equal(txHashes.length);

        const traceFilterResponse = await customDevRpcRequest("trace_filter", [
          {
            fromBlock: blockNumberHex,
            toBlock: blockNumberHex,
          },
        ]);
        const uniqueTxsInTraces = Object.keys(
          traceFilterResponse.reduce(
            (prev, cur) => ({
              ...prev,
              [cur.transactionHash]: true,
            }),
            {}
          )
        );

        console.log(`
          Ethereum transactions count: ${ethBlock.transactions.length},
          Ethereum traces count: ${traceFilterResponse.length},
          Ethereum unique transactions in traces count: ${uniqueTxsInTraces.length}
       `);

        // Assert that all eth transactions were traced
        expect(ethBlock.transactions.length).to.equal(uniqueTxsInTraces.length);
      },
    });
  },
});
