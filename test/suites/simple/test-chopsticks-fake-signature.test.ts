import { afterAll, describe, expect, it } from "vitest";
import {
  setupContext,
  signFake,
  signFakeWithApi,
  testingPairs,
} from "@acala-network/chopsticks-testing";

describe(
  "Chopsticks Fake Signature",
  async () => {
    const { api, dev, teardown } = await setupContext({
      endpoint: ["wss://wss.api.moonbase.moonbeam.network"],
      timeout: 400_000,
    });

    afterAll(async () => await teardown());

    it("accept valid signature", async () => {
      const { alith, baltathar } = testingPairs();
      await dev.setStorage({
        System: {
          Account: [[[alith.address], { providers: 1, data: { free: 1000 * 1e12 } }]],
        },
      });

      const tx = api.tx.balances.transferAllowDeath(baltathar.address, 100);

      await tx.signAsync(alith);

      await expect(tx.send()).resolves.toBeTruthy();
    });

    it("reject invalid signature", async () => {
      const { alith, baltathar } = testingPairs();
      const { nonce } = await api.query.system.account(alith.address);
      const tx = api.tx.balances.transferAllowDeath(baltathar.address, 100);

      tx.signFake(alith.address, {
        nonce,
        genesisHash: api.genesisHash,
        runtimeVersion: api.runtimeVersion,
        blockHash: api.genesisHash,
      });

      await expect(tx.send()).rejects.toThrow('1010: {"invalid":{"badProof":null}}');
    });

    it("accept mock signature (with api)", async () => {
      const { alith, baltathar } = testingPairs();
      await dev.setStorage({
        System: {
          Account: [[[alith.address], { providers: 1, data: { free: 1000 * 1e12 } }]],
        },
      });

      const tx = api.tx.balances.transferAllowDeath(baltathar.address, 100);

      await signFakeWithApi(api, tx, alith.address);

      await expect(tx.send()).resolves.toBeTruthy();
    });

    it("accept mock signature (manually input options)", async () => {
      const { alith, baltathar } = testingPairs();
      await dev.setStorage({
        System: {
          Account: [[[alith.address], { providers: 1, data: { free: 1000 * 1e12 } }]],
        },
      });

      const { nonce } = await api.query.system.account(alith.address);
      const tx = api.tx.balances.transferAllowDeath(baltathar.address, 100);

      signFake(tx, alith.address, {
        nonce,
        genesisHash: api.genesisHash,
        runtimeVersion: api.runtimeVersion,
        blockHash: api.genesisHash,
      });

      await expect(tx.send()).resolves.toBeTruthy();
    });
  },
  { timeout: 120_000 }
);
