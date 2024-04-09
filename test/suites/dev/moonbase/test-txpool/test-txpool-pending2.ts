import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  MIN_GAS_PRICE,
  createEthersTransaction,
  sendRawTransaction,
} from "@moonwall/util";
import { encodeFunctionData, toHex } from "viem";

describeSuite({
  id: "D013908",
  title: "TxPool - Ethereum Contract Call",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let multiplyBy7Contract: `0x${string}`;
    let txHash: `0x${string}`;

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("MultiplyBy7", {
        gas: 1048576n,
      });
      multiplyBy7Contract = contractAddress;
      const data = encodeFunctionData({
        abi,
        functionName: "multiply",
        args: [5],
      });

      const rawTx = await createEthersTransaction(context, {
        to: contractAddress,
        data,
        gasLimit: 12000000n,
        gasPrice: MIN_GAS_PRICE,
      });

      txHash = await sendRawTransaction(context, rawTx);
    });

    it({
      id: "T01",
      title: "should appear in the txpool inspection",
      test: async function () {
        const inspect = (await context
          .viem()
          .transport.request({ method: "txpool_inspect" })) as any;

        const data = inspect.pending[ALITH_ADDRESS.toLowerCase()][toHex(1)];

        expect(data).to.not.be.undefined;
        expect(data).to.be.equal(
          multiplyBy7Contract.toLowerCase() + ": 0 wei + 12000000 gas x 10000000000 wei"
        );
      },
    });

    it({
      id: "T02",
      title: "should appear in the txpool content",
      test: async function () {
        const content = (await context
          .viem()
          .transport.request({ method: "txpool_content" })) as any;
        const data = content.pending[ALITH_ADDRESS.toLowerCase()][toHex(1)];
        expect(data).to.include({
          blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
          blockNumber: null,
          from: ALITH_ADDRESS.toLowerCase(),
          gas: "0xb71b00",
          gasPrice: "0x2540be400",
          hash: txHash,
          nonce: toHex(1),
          to: multiplyBy7Contract.toLowerCase(),
          value: "0x0",
        });
      },
    });
  },
});
