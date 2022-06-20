import { Keyring } from "@polkadot/api";
import { expect } from "chai";
import { alith } from "../../../util/accounts";

import { describeParachain } from "../../../util/setup-para-tests";
import { sendAllStreamAndWaitLast } from "../../../util/transactions";

// This test will run on local until the new runtime is available

const runtimeVersion = "local";
describeParachain(
  `Runtime ${runtimeVersion} migration`,
  {
    parachain: {
      chain: "moonbase-local",
      runtime: "runtime-1200",
      binary: "v0.20.1",
    },
    relaychain: {
      binary: "v0.9.16",
    },
  },
  (context) => {
    it("sxhould fix total counted for staking", async function () {
      // Expected to take 4 blocks to setup + 10 blocks for upgrade + 4 blocks to check =>
      // ~300000 + init 50000 + error marging 150000
      this.timeout(500000);

      const keyring = new Keyring({ type: "ethereum" });

      // verify alith initial total staked
      expect(
        (await context.polkadotApiParaone.query.parachainStaking.candidatePool())
          .find((c) => c.owner.toString() == alith.address)
          .amount.toBigInt()
      ).to.be.equal(1000000000000000000000n);

      const delegatorCount = 3;

      // Creating delegator accounts
      const delegators = await Promise.all(
        new Array(delegatorCount).fill(0).map((_, i) => {
          return keyring.addFromUri(`0x${(i + 20000000).toString().padStart(64, "0")}`);
        })
      );

      const minDelegatorStk = (
        await context.polkadotApiParaone.consts.parachainStaking.minDelegatorStk
      ).toBigInt();

      process.stdout.write(
        `Extrinsic: Transfer ${minDelegatorStk / 10n ** 18n + 2n} tokens to ${
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
            minDelegatorStk + 2n * 10n ** 18n
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

      // verify alith new delegators are added
      expect(
        (await context.polkadotApiParaone.query.parachainStaking.candidatePool())
          .find((c) => c.owner.toString() == alith.address)
          .amount.toBigInt()
      ).to.be.equal(1000000000000000000000n + minDelegatorStk * BigInt(delegatorCount));

      process.stdout.write(
        `Extrinsic: Bonding more (1 token) from` +
          ` ${delegators.length} delegators to trigger the bug...`
      );
      const bondMoreBatches = await Promise.all(
        delegators.map((delegator, index) =>
          context.polkadotApiParaone.tx.parachainStaking
            .delegatorBondMore(alith.address, 1n * 10n ** 18n)
            .signAsync(delegator, { nonce: 1 })
        )
      );

      await sendAllStreamAndWaitLast(context.polkadotApiParaone, bondMoreBatches);
      await context.waitBlocks(1);
      process.stdout.write(`✅: ${bondMoreBatches.length} extrinsics\n`);

      process.stdout.write(`Verifying candidate pool bug pre-migration...`);
      // Verify BUG: alith total didn't increase with the bond more
      expect(
        (await context.polkadotApiParaone.query.parachainStaking.candidatePool())
          .find((c) => c.owner.toString() == alith.address)
          .amount.toBigInt()
      ).to.be.equal(1000000000000000000000n + minDelegatorStk * BigInt(delegatorCount));
      process.stdout.write(`✅\n`);

      await context.upgradeRuntime(alith, "moonbase", runtimeVersion);

      process.stdout.write("Verifying candidate pool is fixed post-migration...");
      expect(
        (await context.polkadotApiParaone.query.parachainStaking.candidatePool())
          .find((c) => c.owner.toString() == alith.address)
          .amount.toBigInt()
      ).to.be.equal(
        1000000000000000000000n + (minDelegatorStk + 1n * 10n ** 18n) * BigInt(delegatorCount)
      );
      process.stdout.write("✅\n");
    });
  }
);
