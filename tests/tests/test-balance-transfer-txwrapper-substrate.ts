import { expect } from "chai";
import { methods as substrateMethods } from "@substrate/txwrapper-substrate";
// import {
//   construct,
//   decode,
//   deriveAddress,
//   getRegistry,
//   methods,
//   PolkadotSS58Format,
// } from "@substrate/txwrapper-polkadot";
import { getRegistry } from "@substrate/txwrapper-registry";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_BALANCE } from "../util/constants";

import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createTransfer, rpcToLocalNode } from "../util/transactions";

describeDevMoonbeam("Balance transfer", (context) => {
  const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
  before("Create block with transfer to test account of 512", async () => {
    // await context.createBlock({
    //   transactions: [await createTransfer(context.web3, TEST_ACCOUNT, 512)],
    // });
    const { block } = await rpcToLocalNode(context.rpcPort, "chain_getBlock");
    const blockHash = await rpcToLocalNode(context.rpcPort, "chain_getBlockHash");
    const genesisHash = await rpcToLocalNode(context.rpcPort, "chain_getBlockHash", [0]);
    const metadataRpc = await rpcToLocalNode(context.rpcPort, "state_getMetadata");
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    // const registry = await context.polkadotApi.registry; //.getChainProperties();
    // const knownTypes = await context.polkadotApi.registry.get();
    // console.log("knownTypes", knownTypes);

    const { specVersion, transactionVersion, specName, chainName } = await rpcToLocalNode(
      context.rpcPort,
      "state_getRuntimeVersion"
    );
    console.log("chainName", chainName);
    console.log("specName", specName);
    console.log("specVersion", specVersion);
    const registry = getRegistry({
      chainName: "Moonriver",
      specName,
      specVersion,
      metadataRpc,
    });
    console.log("REGISTRY", registry.knownTypes);
    console.log("specName2", specName);
    substrateMethods.balances.transfer(
      {
        dest: TEST_ACCOUNT,
        value: 512,
      },
      {
        address: GENESIS_ACCOUNT, // deriveAddress(GENESIS_ACCOUNT, PolkadotSS58Format.polkadot),
        blockHash,
        blockNumber: registry.createType("BlockNumber", block.header.number).toNumber(),
        eraPeriod: 64,
        genesisHash,
        metadataRpc,
        nonce: 0, // Assuming this is Alice's first tx on the chain
        specVersion,
        tip: 0,
        transactionVersion,
      },
      {
        metadataRpc,
        // @ts-ignore
        registry,
      }
    );
  });

  it("should decrease from account", async function () {
    // 21000 covers the cost of the transaction
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (GENESIS_ACCOUNT_BALANCE - 512n - 21000n * 1_000_000_000n).toString()
    );
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
