import "@moonbeam-network/api-augment";
import {
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
  sendRawTransaction,
} from "moonwall";
import { encodeFunctionData, type Abi, parseEther } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { createViemTransaction } from "./helpers";
import { getTransactionReceiptWithRetry } from "../../../../helpers/eth-transactions";

describeSuite({
  id: "D010308",
  title: "EIP-7702 Self-Delegation Test",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let counterAddress: `0x${string}`;
    let counterAbi: Abi;
    let chainId: number;

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

      const counter = await deployCreateCompiledContract(context, "Counter");
      counterAddress = counter.contractAddress;
      counterAbi = counter.abi;
    });

    it({
      id: "T01",
      title: "should test gas cost for self-delegation with correct nonce",
      test: async () => {
        // Check counter was incremented
        const init_count = await context.viem().readContract({
          address: counterAddress,
          abi: counterAbi,
          functionName: "count",
          args: [],
        });
        expect(init_count).toBe(0n);

        const eoaPrivateKey = generatePrivateKey();
        const selfDelegatingEOA = privateKeyToAccount(eoaPrivateKey);

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(selfDelegatingEOA.address, parseEther("5")),
        ]);

        // Self-authorization (EOA delegates to a contract on behalf of itself)
        // In EIP-7702, when the authorizing address is the same as the sender,
        // the authorization nonce should be current_nonce + 1 because the EVM
        // increments the nonce before processing the authorization list
        const currentNonce = await context.viem().getTransactionCount({
          address: selfDelegatingEOA.address,
        });

        console.log(`Self-delegating EOA current nonce: ${currentNonce}`);

        const selfAuth = await selfDelegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: chainId,
          nonce: currentNonce + 1, // current_nonce + 1 for self-authorizing transactions
        });

        console.log(`Authorization created with nonce: ${selfAuth.nonce}`);

        // Transaction sent by the same EOA that signed the authorization
        const selfTx = {
          to: selfDelegatingEOA.address,
          data: encodeFunctionData({
            abi: counterAbi,
            functionName: "increment",
            args: [],
          }),
          nonce: currentNonce, // Current nonce for the transaction
          chainId: chainId,
          authorizationList: [selfAuth],
          txnType: "eip7702" as const,
          privateKey: eoaPrivateKey,
        };

        console.log(`Transaction will be sent with nonce: ${selfTx.nonce}`);

        // Send the self-signed transaction directly
        const signedTx = await createViemTransaction(context, selfTx);
        const hash = await sendRawTransaction(context, signedTx);
        console.log(`Transaction signed, sending to network...`);
        await context.createBlock();

        // Get transaction receipt to check for events and status
        const receipt = await getTransactionReceiptWithRetry(context, hash);

        expect(receipt.status).toBe("success");

        console.log(`Self-delegation gas used: ${receipt.gasUsed}`);

        // Verify delegation was set
        const code = await context.viem().getCode({
          address: selfDelegatingEOA.address,
        });
        expect(code).toBeDefined();
        expect(code?.startsWith("0xef0100")).toBe(true);

        console.log(`Delegation code set: ${code}`);

        // Check counter was incremented
        const count = await context.viem().readContract({
          address: selfDelegatingEOA.address,
          abi: counterAbi,
          functionName: "count",
          args: [],
        });
        expect(count).toBe((init_count as bigint) + 1n);

        console.log(`Counter value through delegation: ${count}`);
        console.log(`✅ Self-delegation test passed!`);
      },
    });
  },
});
