import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { alith } from "../../../util/accounts";

import { describeParachain } from "../../../util/setup-para-tests";
import { sendAllStreamAndWaitLast } from "../../../util/transactions";

// This test will run on local until the new runtime is available

const runtimeVersion = "runtime-1200"; // TODO: replace by `runtime-1200`
describeParachain(
  `Runtime ${runtimeVersion} migration`,
  {
    parachain: {
      chain: "moonbase-local",
      runtime: "runtime-1103",
      binary: "v0.19.1",
    },
    relaychain: {
      binary: "v0.9.13",
    },
  },
  (context) => {
    it("should split candidates in 2", async function () {
      // Expected to take 4 blocks to setup + 10 blocks for upgrade + 4 blocks to check =>
      // ~300000 + init 50000 + error marging 150000
      this.timeout(500000);
      const keyring = new Keyring({ type: "ethereum" });

      const maxTopDelegations = 360;

      // Creating delegator accounts
      const delegators = await Promise.all(
        new Array(maxTopDelegations).fill(0).map((_, i) => {
          return keyring.addFromUri(`0x${(i + 20000000).toString().padStart(64, "0")}`);
        })
      );

      const minDelegatorStk = (
        (await context.polkadotApiParaone.consts.parachainStaking.minDelegatorStk) as any
      ).toBigInt();

      expect(
        await context.polkadotApiParaone.query.parachainStaking.candidateState.entries()
      ).to.be.length(2);

      process.stdout.write(
        `Extrinsic: Transfer ${minDelegatorStk / 10n ** 18n + 1n} tokens to ${
          delegators.length
        } to delegators...`
      );

      let alithNonce = (
        await context.polkadotApiParaone.rpc.system.accountNextIndex(alith.address)
      ).toNumber();
      const transferTxs = await Promise.all(
        delegators.map(async (delegator) => {
          return context.polkadotApiParaone.tx.balances.transfer(
            delegator.address,
            minDelegatorStk + 1n * 10n ** 18n
          );
        })
      );
      await sendAllStreamAndWaitLast(context.polkadotApiParaone, [
        await context.polkadotApiParaone.tx.utility
          .batchAll(transferTxs)
          .signAsync(alith, { nonce: alithNonce++ }),
      ]);
      process.stdout.write(`✅: ${transferTxs.length} transfers\n`);

      process.stdout.write(
        `Extrinsic: Delegate ${minDelegatorStk / 10n ** 18n} tokens from ${
          delegators.length
        } delegators to alith...`
      );

      const bondBatches = await Promise.all(
        delegators.map((delegator, index) =>
          context.polkadotApiParaone.tx.parachainStaking
            .delegate(alith.address, minDelegatorStk, index + 1, 1)
            .signAsync(delegator, { nonce: 0 })
        )
      );
      await sendAllStreamAndWaitLast(context.polkadotApiParaone, bondBatches);
      await context.waitBlocks(1);
      process.stdout.write(`✅: ${bondBatches.length} extrinsics\n`);

      process.stdout.write(`Verifying candidate state pre-migration...`);
      const candidateStatePreMigration = (
        (await context.polkadotApiParaone.query.parachainStaking.candidateState(
          alith.address
        )) as any
      ).unwrap();
      expect(candidateStatePreMigration.delegators).to.be.length(360);
      expect(candidateStatePreMigration.topDelegations).to.be.length(300);
      expect(candidateStatePreMigration.bottomDelegations).to.be.length(60);
      process.stdout.write(`✅: ${candidateStatePreMigration.delegators.length} delegators\n`);

      await context.upgradeRuntime(alith, "moonbase", runtimeVersion);

      process.stdout.write("Checking candidateState post-migration is empty...");
      expect(
        await context.polkadotApiParaone.query.parachainStaking.candidateState.entries()
      ).to.be.length(0);
      process.stdout.write("✅\n");

      process.stdout.write("Checking candidateInfo post-migration...");
      const candidateInfo =
        await context.polkadotApiParaone.query.parachainStaking.candidateInfo.entries();
      expect(candidateInfo).to.be.length(2);
      const topDelegations = (
        (await context.polkadotApiParaone.query.parachainStaking.topDelegations(
          alith.address
        )) as any
      ).unwrap();
      expect(topDelegations.delegations).to.be.length(300);
      const bottomDelegations = (
        (await context.polkadotApiParaone.query.parachainStaking.bottomDelegations(
          alith.address
        )) as any
      ).unwrap();
      expect(bottomDelegations.delegations).to.be.length(50); // new runtime only allow 50 bottom
      process.stdout.write(`✅\n`);

      process.stdout.write("Waiting extra block being produced...");
      await context.waitBlocks(2); // Make sure the new runtime is producing blocks
      process.stdout.write(`✅ total ${context.blockNumber} block produced\n`);
    });
  }
);
