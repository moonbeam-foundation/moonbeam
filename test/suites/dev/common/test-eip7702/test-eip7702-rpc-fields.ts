import "@moonbeam-network/api-augment";
import {
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
  sendRawTransaction,
} from "moonwall";
import { encodeFunctionData, type Abi } from "viem";
import { generatePrivateKey, privateKeyToAccount, signAuthorization } from "viem/accounts";
import { createViemTransaction } from "./helpers";
import { getTransactionReceiptWithRetry } from "../../../../helpers/eth-transactions";
import { recoverAuthorizationAddress, verifyAuthorization } from "viem/utils";

describeSuite({
  id: "D010307",
  title: "EIP-7702 RPC Field Naming",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let contractAddress: `0x${string}`;
    let contractAbi: Abi;
    let chainId: number;

    beforeAll(async () => {
      // Get the chainId from the RPC
      chainId = await context.viem().getChainId();

      // Deploy the delegation contract
      const { contractAddress: address, abi } = await deployCreateCompiledContract(
        context,
        "BalanceTracker"
      );

      expect(address).toBeTruthy();
      console.log(`Delegation contract deployed at: ${address}`);

      contractAddress = address;
      contractAbi = abi;
    });

    it({
      id: "T01",
      title:
        "should return correctly formatted authorization list fields in eth_getTransactionByHash",
      test: async () => {
        // Create a new EOA for delegation
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);
        const delegatingAddress = delegatingEOA.address;
        console.log(`Created EOA for delegation: ${delegatingAddress}`);

        // Fund the delegating EOA with some balance
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingAddress, 1000000000000000000n),
        ]);

        // Set up delegation authorization
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
          nonce: 0,
        });

        console.log(`Authorization created with signature:`, {
          r: authorization.r,
          s: authorization.s,
          yParity: authorization.yParity,
        });

        const authorizationList = [authorization];

        // Create transaction data
        const targetAddress = "0x1234567890123456789012345678901234567890" as `0x${string}`;
        const targetBalance = 5000n;

        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: [targetAddress, targetBalance],
        });

        // Create EIP-7702 transaction
        const transaction = {
          to: delegatingAddress,
          data: callData,
          chainId: chainId,
          authorizationList,
          txnType: "eip7702" as const,
        };

        console.log(`Creating and sending EIP-7702 transaction...`);

        // Sign and send the transaction
        const signature = await createViemTransaction(context, transaction);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        console.log(`Transaction hash: ${hash}`);

        // Check transaction receipt
        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");

        // Fetch the transaction via RPC and check the authorization list fields
        const tx = await context.viem().getTransaction({ hash });

        console.log(`Transaction type: ${tx.type}`);
        console.log(`Authorization list:`, tx.authorizationList);

        // Verify the transaction type
        expect(tx.type).toBe("eip7702");

        // Verify authorizationList exists and has at least one entry
        expect(tx.authorizationList).toBeDefined();
        expect(tx.authorizationList).toHaveLength(1);

        const auth = tx.authorizationList![0];

        // Test the field naming according to the Ethereum specification
        // These fields should be in camelCase and at the top level, not nested in "signature"

        console.log(`Checking authorization fields...`);
        console.log(`Authorization object:`, auth);

        // Check that chainId exists and is in camelCase (not chain_id)
        expect(auth).toHaveProperty("chainId");
        expect(auth.chainId).toBe(chainId);

        // This should FAIL with the current implementation (chain_id instead of chainId)
        expect(auth).not.toHaveProperty("chain_id");

        // Check that address exists
        expect(auth).toHaveProperty("address");
        expect(auth.address.toLowerCase()).toBe(contractAddress.toLowerCase());

        // Check that nonce exists
        expect(auth).toHaveProperty("nonce");

        // Check that yParity, r, and s are at the top level (not nested in a "signature" object)
        expect(auth).toHaveProperty("yParity");
        expect(auth).toHaveProperty("r");
        expect(auth).toHaveProperty("s");

        // These should FAIL with the current implementation (fields are nested in signature object)
        expect(auth).not.toHaveProperty("signature");

        // Verify the signature values match what we sent
        // INFO: Trimming leading zeros is necessary because of: https://github.com/wevm/viem/pull/3455
        expect(auth.r).toBe(
          authorization.r.replace(/^0x0+/, "0x") /* Trim leading zeros for comparison */
        );
        expect(auth.s).toBe(
          authorization.s.replace(/^0x0+/, "0x") /* Trim leading zeros for comparison */
        );
        const valid = await verifyAuthorization({
          address: delegatingAddress,
          authorization: auth,
        });
        expect(valid, `Authorization signature is invalid`).toBeTruthy();

        // yParity can be 0 or 1, but should match the authorization
        // Note: yParity is a number (0 or 1), not a boolean
        expect(auth.yParity).toBe(authorization.yParity);

        console.log(`✅ All authorization list fields are correctly named and structured!`);
      },
    });

    it({
      id: "T02",
      title: "should return correctly formatted authorization list fields in eth_getBlockByNumber",
      test: async () => {
        // Create a new EOA for delegation
        const privateKey = generatePrivateKey();
        const delegatingEOA = privateKeyToAccount(privateKey);
        const delegatingAddress = delegatingEOA.address;

        // Fund the delegating EOA
        await context.createBlock([
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(delegatingAddress, 1000000000000000000n),
        ]);

        // Set up delegation authorization
        const authorization = await delegatingEOA.signAuthorization({
          contractAddress: contractAddress,
          chainId: chainId,
          nonce: 0,
        });

        const authorizationList = [authorization];

        // Create transaction data
        const callData = encodeFunctionData({
          abi: contractAbi,
          functionName: "setBalance",
          args: ["0x1234567890123456789012345678901234567890", 5000n],
        });

        // Create EIP-7702 transaction
        const transaction = {
          to: delegatingAddress,
          data: callData,
          chainId: chainId,
          authorizationList,
          txnType: "eip7702" as const,
        };

        // Sign and send the transaction
        const signature = await createViemTransaction(context, transaction);
        const hash = await sendRawTransaction(context, signature);
        await context.createBlock();

        const receipt = await getTransactionReceiptWithRetry(context, hash);
        expect(receipt.status).toBe("success");

        // Fetch the block containing the transaction
        const block = await context.viem().getBlock({
          blockNumber: receipt.blockNumber,
          includeTransactions: true,
        });

        console.log(`Block retrieved with ${block.transactions.length} transactions`);

        // Find our transaction in the block
        const txInBlock = block.transactions.find((t: any) => t.hash === hash);
        expect(txInBlock).toBeDefined();

        console.log(`Transaction in block found with type: ${(txInBlock as any).type}`);
        console.log(`Authorization list in block:`, (txInBlock as any).authorizationList);

        // Verify the authorization list fields in the block response
        const authList = (txInBlock as any).authorizationList;
        expect(authList).toBeDefined();
        expect(authList).toHaveLength(1);

        const auth = authList[0];

        // Same checks as T01 - fields should be properly named in block responses too
        expect(auth).toHaveProperty("chainId");
        expect(auth).not.toHaveProperty("chain_id");

        expect(auth).toHaveProperty("yParity");
        expect(auth).toHaveProperty("r");
        expect(auth).toHaveProperty("s");
        expect(auth).not.toHaveProperty("signature");

        console.log(
          `✅ Authorization list fields in block response are correctly named and structured!`
        );
      },
    });
  },
});
