import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { encodeFunctionData, numberToHex, type Abi } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { expectOk } from "../../../../helpers";

describeSuite({
  id: "D010301",
  title: "EIP-7702 Transactions",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contractAddress: `0x${string}`;
    let contractAbi: Abi;

    beforeAll(async () => {
      // Deploy the delegation contract
      const { contractAddress: address, abi } = await deployCreateCompiledContract(
        context,
        "EIP7702Delegation"
      );

      expect(address).toBeTruthy();
      console.log(`Delegation contract deployed at: ${address}`);

      contractAddress = address;
      contractAbi = abi;
    });

    it({
      id: "T01",
      title: "happy path - should successfully delegate with valid EIP-7702 authorization",
      test: async () => {
        // Create a new EOA for delegation
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);

        const delegatingAddress = delegatingEOA.address;

        console.log(`Created EOA for delegation: ${delegatingAddress}`);

        // Fund the delegating EOA with some balance from ALITH
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingAddress, 1000000000000000000n),
        ]);

        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: 1281,
          nonce: 0,
        });

        console.log(
          `Authorization created for ${delegatingAddress} to delegate to ${contractAddress}`
        );
        console.log(`Authorization nonce: ${authorization.nonce}`);
        console.log(`Authorization details:`, {
          contractAddress: contractAddress,
          chainId: authorization.chainId,
          nonce: authorization.nonce?.toString(),
          r: authorization.r,
          s: authorization.s,
          yParity: authorization.yParity,
        });

        // Create the authorization list
        const authorizationList = [authorization];

        // Use the delegation ABI from helpers

        // Set balance for an arbitrary address
        const targetAddress = "0x1234567890123456789012345678901234567890" as `0x${string}`;
        const targetBalance = 5000n;

        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: [targetAddress, targetBalance],
        });

        // Create a raw EIP-7702 transaction manually
        console.log(`Creating EIP-7702 transaction with authorizationList...`);
        console.log(`Authorization list being sent:`, authorizationList);

        // Get ALITH's account info for signing
        const alithPrivateKey =
          "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
        const alithAccount = privateKeyToAccount(alithPrivateKey);

        // Create the transaction object with authorizationList
        const transaction = {
          to: delegatingAddress,
          data: callData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: 0n,
          nonce: await context.viem("public").getTransactionCount({
            address: alithAccount.address,
          }),
          chainId: 1281,
          authorizationList: authorizationList,
          type: "eip7702" as const,
        };

        console.log(`Transaction object:`, transaction);

        // Sign the transaction
        const signature = await alithAccount.signTransaction(transaction);
        console.log(`Signed transaction: ${signature}`);

        const result = await context.createBlock(signature);
        console.log(`Transaction submitted by ALITH for delegation to ${delegatingAddress}`);
        console.log(`Block result:`, result.result);
        console.log(`Result object keys:`, Object.keys(result));
        console.log(`Full result:`, result);

        // Try to get transaction hash from different sources
        let txHash: `0x${string}` | undefined;
        if (result.hash) {
          txHash = result.hash as `0x${string}`;
        } else if (result.result?.hash) {
          txHash = result.result.hash as `0x${string}`;
        } else if (result.result?.extrinsic?.hash) {
          txHash = result.result.extrinsic.hash.toHex() as `0x${string}`;
        }

        console.log(`Transaction hash: ${txHash}`);

        if (txHash) {
          // Check transaction receipt
          const receipt = await context.viem("public").getTransactionReceipt({
            hash: txHash,
          });
          console.log(`Transaction receipt status: ${receipt.status}`);
          console.log(`Transaction receipt logs:`, receipt.logs);

          // Check the transaction details
          const tx = await context.viem("public").getTransaction({
            hash: txHash,
          });
          console.log(`Transaction type: ${tx.type}`);
          console.log(`Transaction authorizationList:`, tx.authorizationList);

          // Also check the raw transaction
          console.log(`Raw transaction:`, tx);
        } else {
          console.log(`WARNING: Could not find transaction hash in result`);
        }

        // Check if the delegating address now has delegated code
        const codeAtDelegator = await context.viem("public").getCode({
          address: delegatingAddress,
        });
        console.log(`Code at delegator address ${delegatingAddress}: ${codeAtDelegator}`);

        // Also check code at contract address for comparison
        const codeAtContract = await context.viem("public").getCode({
          address: contractAddress,
        });
        console.log(
          `Code at contract address ${contractAddress}: ${codeAtContract?.slice(0, 50)}...`
        );

        // EIP-7702 sets a special delegated code format: 0xef0100 + 20-byte address
        expect(codeAtDelegator).toBeTruthy();
        expect(codeAtDelegator?.startsWith("0xef0100")).toBe(true);
        expect(codeAtDelegator?.length).toBe(48); // 0x + ef0100 (6) + address (40)

        // Now check if the delegation worked
        // The storage should be in Baltathar's account context, not the contract's

        // Calculate storage slot for mapping(address => uint256) balances
        // slot = keccak256(abi.encode(targetAddress, 0))
        const { keccak256, concat } = await import("viem");
        const storageSlot = keccak256(
          concat([
            targetAddress.toLowerCase().padEnd(66, "0") as `0x${string}`,
            numberToHex(0n, { size: 32 }),
          ])
        );

        // Check storage at the delegating EOA's address
        const storageAtDelegator = await context.viem("public").getStorageAt({
          address: delegatingAddress,
          slot: storageSlot,
        });

        const actualBalance = BigInt(storageAtDelegator || "0");
        console.log(`Storage at delegating address ${delegatingAddress}: ${actualBalance}`);

        // Also check the contract storage (should be 0 if delegation worked properly)
        const contractStorageBalance = await context.viem("public").readContract({
          address: contractAddress!,
          abi: contractAbi,
          functionName: "getBalance",
          args: [targetAddress],
        });

        console.log(`Balance in contract storage: ${contractStorageBalance}`);

        // Let's check if we can read the balance through the delegated address
        try {
          const delegatedBalance = await context.viem("public").readContract({
            address: delegatingAddress,
            abi: contractAbi,
            functionName: "getBalance",
            args: [targetAddress],
          });
          console.log(`Balance read through delegated address: ${delegatedBalance}`);
        } catch (error) {
          console.log(`Error reading through delegated address:`, error);
        }

        // Happy path expectations for EIP-7702
        // The storage is NOT in the delegating address, but accessed through the contract
        // The delegating address should have the delegation code
        expect(codeAtDelegator).toBeTruthy();
        expect(codeAtDelegator?.startsWith("0xef0100")).toBe(true);

        // Reading through the delegated address should return the correct balance
        const delegatedBalance = await context.viem("public").readContract({
          address: delegatingAddress,
          abi: contractAbi,
          functionName: "getBalance",
          args: [targetAddress],
        });

        expect(delegatedBalance).to.equal(targetBalance);
        console.log(
          `SUCCESS: EIP-7702 delegation worked! Balance ${delegatedBalance} can be read through the delegating address`
        );

        // Additional test: call incrementBalance to verify continued delegation
        const incrementData = encodeFunctionData({
          abi: contractAbi,
          functionName: "incrementBalance",
          args: [targetAddress, 500n],
        });

        // Second transaction: increment balance through the delegated address
        // We don't need to send the authorization again since it's already set
        const incrementTx = {
          to: delegatingAddress,
          data: incrementData,
          gas: 200000n,
          maxFeePerGas: 10_000_000_000n,
          maxPriorityFeePerGas: 0n,
          nonce: await context.viem("public").getTransactionCount({
            address: alithAccount.address,
          }),
          chainId: 1281,
        };

        const signedIncrement = await alithAccount.signTransaction(incrementTx);
        await expectOk(context.createBlock(signedIncrement));

        // Check updated balance through the delegated address
        const updatedBalance = await context.viem("public").readContract({
          address: delegatingAddress,
          abi: contractAbi,
          functionName: "getBalance",
          args: [targetAddress],
        });

        expect(updatedBalance).to.equal(5500n);

        console.log(`After increment: Balance is now ${updatedBalance}`);
        console.log(`EIP-7702 delegation is working correctly!`);
      },
    });
  },
});
