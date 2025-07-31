import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { encodeFunctionData, numberToHex, type Abi } from "viem";
import { privateKeyToAccount } from "viem/accounts";
import { createEIP7702Authorization, expectOk } from "../../../../helpers";

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
        const delegatingEOA = privateKeyToAccount(
          "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133"
        );
        const delegatingAddress = delegatingEOA.address;

        console.log(`Created EOA for delegation: ${delegatingAddress}`);

        // Fund the delegating EOA with some balance from ALITH
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingAddress, 1000000000000000000n),
        ]);

        // Get the actual nonce of the delegating EOA
        const delegatingNonce = await context
          .viem("public")
          .getTransactionCount({ address: delegatingAddress });

        // Create authorization for the new EOA to delegate to the contract
        const authorization = await createEIP7702Authorization(
          delegatingEOA,
          1281n, // chainId
          BigInt(delegatingNonce), // Use actual nonce
          contractAddress!
        );

        console.log(
          `Authorization created for ${delegatingAddress} to delegate to ${contractAddress}`
        );
        console.log(`Authorization nonce: ${authorization.nonce}`);

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

        // Try using createEthersTransaction with proper authorization
        const rawSigned = await createEthersTransaction(context, {
          to: delegatingAddress,
          data: callData,
          gasLimit: 200000,
          authorizationList: authorizationList,
        });

        const result = await context.createBlock(rawSigned);
        console.log(`Transaction submitted by ALITH for delegation to ${delegatingAddress}`);
        console.log(`Block result:`, result.result);

        // Check if the delegating address now has delegated code
        const codeAtDelegator = await context.viem("public").getCode({
          address: delegatingAddress,
        });

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

        // Happy path expectations
        expect(actualBalance).to.equal(targetBalance);
        console.log(
          `SUCCESS: EIP-7702 delegation worked! Balance ${actualBalance} was stored in the delegating account's storage`
        );

        // Additional test: call incrementBalance to verify continued delegation
        const incrementData = encodeFunctionData({
          abi: contractAbi,
          functionName: "incrementBalance",
          args: [targetAddress, 500n],
        });

        // Second transaction: increment balance using the same delegation
        const rawSigned2 = await createEthersTransaction(context, {
          to: delegatingAddress,
          data: incrementData,
          gasLimit: 200000,
          authorizationList: [authorization],
        });

        await expectOk(context.createBlock(rawSigned2));

        // Check updated balance
        const updatedStorage = await context.viem("public").getStorageAt({
          address: delegatingAddress,
          slot: storageSlot,
        });

        const updatedBalance = BigInt(updatedStorage || "0");
        expect(updatedBalance).to.equal(5500n);

        console.log(`After increment: Balance is now ${updatedBalance}`);
        console.log(`EIP-7702 delegation is working correctly!`);
      },
    });
  },
});
