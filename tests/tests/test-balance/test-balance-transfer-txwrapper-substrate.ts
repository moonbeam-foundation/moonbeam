// As inspired by https://github.com/paritytech/txwrapper/blob/master/examples/polkadot.ts
// This flow is used by some exchange partners like kraken
import "@moonbeam-network/api-augment";

import { EXTRINSIC_VERSION } from "@polkadot/types/extrinsic/v4/Extrinsic";
import {
  createMetadata,
  getSpecTypes,
  KeyringPair,
  OptionsWithMeta,
  TypeRegistry,
} from "@substrate/txwrapper-core";
import { createSignedTx, createSigningPayload } from "@substrate/txwrapper-core/lib/core/construct";
import { getRegistryBase } from "@substrate/txwrapper-core/lib/core/metadata";
import { methods as substrateMethods } from "@substrate/txwrapper-substrate";
import { expect } from "chai";

import { alith, ALITH_GENESIS_LOCK_BALANCE, generateKeyringPair } from "../../util/accounts";
import { verifyLatestBlockFees } from "../../util/block";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { rpcToLocalNode } from "../../util/transactions";

/**
 * Signing function. Implement this on the OFFLINE signing device.
 *
 * @param pair - The signing pair.
 * @param signingPayload - Payload to sign.
 */
export function signWith(
  pair: KeyringPair,
  signingPayload: string,
  options: OptionsWithMeta
): `0x${string}` {
  const { registry, metadataRpc } = options;
  // Important! The registry needs to be updated with latest metadata, so make
  // sure to run `registry.setMetadata(metadata)` before signing.
  registry.setMetadata(createMetadata(registry, metadataRpc));

  const { signature } = registry
    .createType("ExtrinsicPayload", signingPayload, {
      version: EXTRINSIC_VERSION,
    })
    .sign(pair);

  return signature as `0x${string}`; //TODO: fix this when type problem is fixed
}

describeDevMoonbeam("Balance transfer - txwrapper", (context) => {
  const randomAccount = generateKeyringPair();
  before("Create block with transfer to test account of 512", async function () {
    // txwrapper takes more time to initiate :/
    this.timeout(10000);

    const [
      { block },
      blockHash,
      genesisHash,
      metadataRpc,
      { specVersion, transactionVersion, specName },
    ] = await Promise.all([
      rpcToLocalNode(context.rpcPort, "chain_getBlock"),
      rpcToLocalNode(context.rpcPort, "chain_getBlockHash"),
      rpcToLocalNode(context.rpcPort, "chain_getBlockHash", [0]),
      rpcToLocalNode(context.rpcPort, "state_getMetadata"),
      rpcToLocalNode(context.rpcPort, "state_getRuntimeVersion"),
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

    const unsigned = substrateMethods.balances.transfer(
      {
        dest: randomAccount.address,
        value: 512,
      },
      {
        address: alith.address,
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

    await rpcToLocalNode(context.rpcPort, "author_submitExtrinsic", [tx]);
    await context.createBlock();
  });

  it("should increase to account", async function () {
    expect(await context.web3.eth.getBalance(randomAccount.address, 0)).to.equal("0");
    expect(await context.web3.eth.getBalance(randomAccount.address, 1)).to.equal("512");
  });

  it("should reflect balance identically on polkadot/web3", async function () {
    const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const balance = await (
      await context.polkadotApi.at(block1Hash)
    ).query.system.account(alith.address);
    expect(await context.web3.eth.getBalance(alith.address, 1)).to.equal(
      (balance.data.free.toBigInt() - ALITH_GENESIS_LOCK_BALANCE).toString()
    );
  });
  it("should check fees", async function () {
    await verifyLatestBlockFees(context, BigInt(512));
  });
});
