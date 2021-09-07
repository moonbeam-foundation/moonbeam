// As inspired by https://github.com/paritytech/txwrapper/blob/master/examples/polkadot.ts
// This flow is used by some exchange partners like kraken

import { expect } from "chai";
import { methods as substrateMethods } from "@substrate/txwrapper-substrate";
import {
  createSignedTx,
  createSigningPayload,
  KeyringPair,
  OptionsWithMeta,
} from "@substrate/txwrapper";
import { Keyring } from "@polkadot/api";
import { getRegistry } from "@substrate/txwrapper-registry";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { rpcToLocalNode } from "../util/transactions";
import { createMetadata } from "@substrate/txwrapper/lib/util";
import { EXTRINSIC_VERSION } from "@polkadot/types/extrinsic/v4/Extrinsic";

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
): string {
  const { registry, metadataRpc } = options;
  // Important! The registry needs to be updated with latest metadata, so make
  // sure to run `registry.setMetadata(metadata)` before signing.
  registry.setMetadata(createMetadata(registry, metadataRpc));

  const { signature } = registry
    .createType("ExtrinsicPayload", signingPayload, {
      version: EXTRINSIC_VERSION,
    })
    .sign(pair);

  return signature;
}

describeDevMoonbeam("Balance transfer", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  before("Create block with transfer to test account of 512", async () => {
    const { block } = await rpcToLocalNode(context.rpcPort, "chain_getBlock");
    const blockHash = await rpcToLocalNode(context.rpcPort, "chain_getBlockHash");
    const genesisHash = await rpcToLocalNode(context.rpcPort, "chain_getBlockHash", [0]);
    const metadataRpc = await rpcToLocalNode(context.rpcPort, "state_getMetadata");

    const { specVersion, transactionVersion, specName } = await rpcToLocalNode(
      context.rpcPort,
      "state_getRuntimeVersion"
    );
    const registry = getRegistry({
      chainName: "Moonriver",
      specName,
      specVersion,
      metadataRpc,
    });
    const unsigned = substrateMethods.balances.transfer(
      {
        dest: TEST_ACCOUNT,
        value: 512,
      },
      {
        address: GENESIS_ACCOUNT,
        blockHash,
        blockNumber: registry.createType("BlockNumber", block.header.number).toNumber(),
        eraPeriod: 64,
        genesisHash,
        metadataRpc,
        nonce: 0, // Assuming this is Gerald's first tx on the chain
        specVersion,
        tip: 0,
        transactionVersion,
      },
      {
        metadataRpc,
        // @ts-ignore // looks like txwrpper doesnt have the latest versions
        registry,
      }
    );
    //@ts-ignore
    const signingPayload = createSigningPayload(unsigned, { registry });
    const keyring = new Keyring({ type: "ethereum" });
    const genesis = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    const signature = signWith(genesis, signingPayload, {
      metadataRpc,
      //@ts-ignore
      registry,
    });
    // Serialize a signed transaction.
    //@ts-ignore
    const tx = createSignedTx(unsigned, signature, { metadataRpc, registry });

    await rpcToLocalNode(context.rpcPort, "author_submitExtrinsic", [tx]);
    await context.createBlock();
  });

  it("should increase to account", async function () {
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 0)).to.equal("0");
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal("512");
  });

  it("should reflect balance identically on polkadot/web3", async function () {
    const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (
        await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)
      ).data.free.toString()
    );
  });
});
