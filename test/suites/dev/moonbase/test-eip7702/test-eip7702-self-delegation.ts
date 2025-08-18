import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, type Abi, parseEther, parseGwei } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D020806",
  title: "EIP-7702 Self-Delegation Test",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let counterAddress: `0x${string}`;
    let counterAbi: Abi;

    beforeAll(async () => {
      const counter = await deployCreateCompiledContract(context, "Counter");
      counterAddress = counter.contractAddress;
      counterAbi = counter.abi;
    });

    it({
      id: "T01",
      title: "should test gas cost for self-delegation with correct nonce",
      test: async () => {
        // Check counter was incremented
        const init_count = await context.viem("public").readContract({
          address: counterAddress,
          abi: counterAbi,
          functionName: "count",
          args: [],
        });
        expect(init_count).toBe(0n);

        const selfDelegatingEOA = privateKeyToAccount(generatePrivateKey());

        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(selfDelegatingEOA.address, parseEther("5")),
        ]);

        // Self-authorization (EOA delegates to a contract on behalf of itself)
        // In EIP-7702, when the authorizing address is the same as the sender,
        // the authorization nonce should be current_nonce + 1 because the EVM
        // increments the nonce before processing the authorization list
        const currentNonce = await context.viem("public").getTransactionCount({
          address: selfDelegatingEOA.address,
        });

        console.log(`Self-delegating EOA current nonce: ${currentNonce}`);

        const selfAuth = await selfDelegatingEOA.signAuthorization({
          contractAddress: counterAddress,
          chainId: 1281,
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
          gas: 200000n,
          maxFeePerGas: parseGwei("10"),
          maxPriorityFeePerGas: parseGwei("1"),
          nonce: currentNonce, // Current nonce for the transaction
          chainId: 1281,
          authorizationList: [selfAuth],
          type: "eip7702" as const,
        };

        console.log(`Transaction will be sent with nonce: ${selfTx.nonce}`);

        // Sign with the same account that created the authorization
        const signature = await selfDelegatingEOA.signTransaction(selfTx);

        console.log(`Transaction signed, sending to network...`);

        // Send the self-signed transaction directly
        const { result } = await context.createBlock(signature);
        const receipt = await context.viem("public").getTransactionReceipt({
          hash: result?.hash as `0x${string}`,
        });
        expect(receipt.status).toBe("success");

        console.log(`Self-delegation gas used: ${receipt.gasUsed}`);

        // Verify delegation was set
        const code = await context.viem("public").getCode({
          address: selfDelegatingEOA.address,
        });
        expect(code).toBeDefined();
        expect(code?.startsWith("0xef0100")).toBe(true);

        console.log(`Delegation code set: ${code}`);

        // Check counter was incremented
        const count = await context.viem("public").readContract({
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
