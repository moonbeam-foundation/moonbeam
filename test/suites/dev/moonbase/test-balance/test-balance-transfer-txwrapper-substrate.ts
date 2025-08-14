// // As inspired by https://github.com/paritytech/txwrapper/blob/master/examples/polkadot.ts
// // This flow is used by some exchange partners like kraken
import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, alith, checkBalance } from "@moonwall/util";
import { TypeRegistry, getSpecTypes } from "@substrate/txwrapper-core";
import { createSignedTx, createSigningPayload } from "@substrate/txwrapper-core/lib/core/construct";
import { getRegistryBase } from "@substrate/txwrapper-core/lib/core/metadata";
import { methods as substrateMethods } from "@substrate/txwrapper-substrate";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { verifyLatestBlockFees, signWith } from "../../../../helpers";

describeSuite({
  id: "D020305",
  title: "Balance transfer - TxWrapper",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let randomAddress: `0x${string}`;

    beforeAll(async () => {
      const privateKey = generatePrivateKey();
      randomAddress = privateKeyToAccount(privateKey).address;
      await context.createBlock();
      const [
        { block },
        blockHash,
        genesisHash,
        metadataRpc,
        { specVersion, transactionVersion, specName },
      ] = await Promise.all([
        customDevRpcRequest("chain_getBlock"),
        customDevRpcRequest("chain_getBlockHash"),
        customDevRpcRequest("chain_getBlockHash", [0]),
        customDevRpcRequest("state_getMetadata"),
        customDevRpcRequest("state_getRuntimeVersion"),
      ]);

      const registry = getRegistryBase({
        chainProperties: {
          ss58Format: 1285,
          tokenDecimals: 18,
          tokenSymbol: "MOVR",
        },
        specTypes: getSpecTypes(new TypeRegistry(), "Moonriver", specName, specVersion),
        metadataRpc,
      });

      const unsigned = substrateMethods.balances.transferAllowDeath(
        {
          dest: randomAddress as any,
          value: 512,
        },
        {
          address: ALITH_ADDRESS,
          blockHash,
          blockNumber: registry.createType("BlockNumber", block.header.number).toNumber(),
          eraPeriod: 64,
          genesisHash,
          metadataRpc,
          nonce: 0, // Assuming this is Alith's first tx on the chain
          specVersion,
          tip: 0,
          transactionVersion,
        },
        {
          metadataRpc,
          registry,
        }
      );

      const signingPayload = createSigningPayload(unsigned, { registry });
      const signature = signWith(alith, signingPayload, {
        metadataRpc,
        registry,
      });
      // Serialize a signed transaction.
      const tx = createSignedTx(unsigned, signature, { metadataRpc, registry });

      await customDevRpcRequest("author_submitExtrinsic", [tx]);
      await context.createBlock();
    }, 60000);

    it({
      id: "T01",
      title: "should show the reducible balanced when some amount is locked",
      test: async function () {
        expect(await checkBalance(context, randomAddress, 1n)).toBe(0n);
        expect(await checkBalance(context, randomAddress, 2n)).toBe(512n);
      },
    });

    it({
      id: "T02",
      title: "should reflect balance identically on polkadot/web3",
      test: async function () {
        const balance = await context.polkadotJs().query.system.account(ALITH_ADDRESS);
        expect(await context.viem().getBalance({ address: ALITH_ADDRESS })).to.equal(
          balance.data.free.toBigInt() +
            balance.data.reserved.toBigInt() -
            balance.data.frozen.toBigInt()
        );
      },
    });

    it({
      id: "T03",
      title: "should check fees",
      test: async function () {
        await verifyLatestBlockFees(context, 512n);
      },
    });
  },
});
