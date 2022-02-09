import { Keyring } from "@polkadot/api";
import { expect } from "chai";
import Web3 from "web3";

import { ALITH_PRIV_KEY, BALTATHAR_PRIV_KEY } from "../../../util/constants";
import { customWeb3Request } from "../../../util/providers";
import { describeParachain } from "../../../util/setup-para-tests";
import { createTransfer } from "../../../util/transactions";

// This test will run on local until the new runtime is available

const runtimeVersion = "runtime-1200";
describeParachain(
  `Runtime ${runtimeVersion} migration`,
  { chain: "moonbase-local", runtime: "runtime-1103", binary: "local" },
  (context) => {
    it("have proper parent hash", async function () {
      // Expected to take 4 blocks to setup + 10 blocks for upgrade + 4 blocks to check =>
      // ~300000 + init 50000 + error marging 150000
      this.timeout(500000);

      const keyring = new Keyring({ type: "ethereum" });
      const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
      const baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

      let baltatharNonce = await context.web3.eth.getTransactionCount(baltathar.address);

      // It takes 10 blocks
      let hasMoreBlockPassed = false;
      const runtimePromise = context
        .upgradeRuntime(alith, "moonbase", runtimeVersion)
        .then(async (blockNumber) => {
          context.waitBlocks(3).then(() => {
            hasMoreBlockPassed = true;
          });
          return blockNumber;
        });

      // It takes 5 blocks for the runtime, however we need to send before to have
      // the extrinsics included

      while (!hasMoreBlockPassed) {
        console.log(`send tx`);
        const tx = await context.web3.eth.accounts.signTransaction(
          {
            from: baltathar.address,
            to: alith.address,
            value: Web3.utils.toWei("1", "ether"),
            gasPrice: Web3.utils.toWei("1", "Gwei"),
            gas: "0x100000",
            nonce: baltatharNonce++,
          },
          BALTATHAR_PRIV_KEY
        );

        await customWeb3Request(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
        await new Promise((resolve) => setTimeout(resolve, 12000));
      }

      for (let i = 1; i < context.blockNumber - 1; i++) {
        console.log(
          `#${i} ${(await context.web3.eth.getBlock(i)).parentHash} ${
            (
              await context.polkadotApiParaone.rpc.state.getRuntimeVersion(
                await context.polkadotApiParaone.rpc.chain.getBlockHash(i)
              )
            ).specVersion
          } (${
            (
              await context.polkadotApiParaone.rpc.chain.getBlock(
                await context.polkadotApiParaone.rpc.chain.getBlockHash(i)
              )
            ).block.extrinsics.length
          } ext)`
        );
      }
      process.stdout.write(`!!!!! \n`);

      expect((await context.web3.eth.getBlock((await runtimePromise) + 1)).parentHash).to.be.string(
        "0x0000000000000000000000000000000000000000000000000000000000000000"
      ); // new runtime only allow 50 bottom
      process.stdout.write(`✅\n`);

      process.stdout.write("Waiting extra block being produced...");
      await context.waitBlocks(2); // Make sure the new runtime is producing blocks
      process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);
    });
  }
);
