import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, DevModeContext, expect, ExtrinsicCreation } from "@moonwall/cli";
import { RUNTIME_CONSTANTS } from "../../helpers";
import { ApiPromise, Keyring } from "@polkadot/api";
import fs from "fs/promises";
import { u8aToHex } from "@polkadot/util";
import { KeyringPair } from "@moonwall/util";

const executeAndVerify = async (context: DevModeContext, txs: any[], callback: (result?: ExtrinsicCreation[]) => Promise<any>) => {
  const { result } = await context.createBlock(txs, { finalize: false });
  await callback(result);
}

describeSuite({
  id: "LD02",
  title: "Lazy Loading - Replay block",
  foundationMethods: "dev",
  options: {
    forkConfig: {
      url: process.env.FORK_URL ?? "https://moonbeam.kaki.dev",
      stateOverridePath: "tmp/lazyLoadingStateOverrides.json",
      verbose: true,
    },
  },
  testCases: ({ it, context, log }) => {
    let api: ApiPromise;
    let fromAccount: KeyringPair;

    beforeAll(async () => {
      api = context.polkadotJs();

      const keyring = new Keyring({ type: "ethereum" });
      if (!process.env.TEST_ACCOUNT_KEY) {
        throw new Error("Missing TEST_ACCOUNT_KEY env variable");
      }
      fromAccount = await keyring.addFromUri(process.env.TEST_ACCOUNT_KEY);

      const runtimeChain = api.runtimeChain.toUpperCase();
      const runtime = runtimeChain
        .split(" ")
        .filter((v) => Object.keys(RUNTIME_CONSTANTS).includes(v))
        .join()
        .toLowerCase();
      const wasmPath = `../target/release/wbuild/${runtime}-runtime/${runtime}_runtime.compact.compressed.wasm`; // editorconfig-checker-disable-line

      const runtimeWasmHex = u8aToHex(await fs.readFile(wasmPath));

      const rtBefore = api.consts.system.version.specVersion.toNumber();
      log("Current runtime:", rtBefore);
      log("About to upgrade to runtime at:", wasmPath);

      await context.createBlock([], { finalize: false });
      const { result } = await context.createBlock(
        api.tx.system.applyAuthorizedUpgrade(runtimeWasmHex),
        { finalize: false }
      );

      // This next block will receive the GoAhead signal
      await context.createBlock([], { finalize: false });
      // The next block will process the runtime upgrade
      await context.createBlock([], { finalize: false });

      // Pre check for runtime upgrade
      const rtAfter = api.consts.system.version.specVersion.toNumber();
      log(`RT upgrade has increased specVersion from ${rtBefore} to ${rtAfter}`);
      expect(rtBefore).to.be.not.equal(rtAfter, "Runtime upgrade failed");
      const specName = api.consts.system.version.specName.toString();
      log(`Currently connected to chain: ${specName}`);

    });

    it({
      id: "T01",
      title: "Can submit remark",
      test: async function () {
        await executeAndVerify(context, [(await api.tx.system.remark("Runtime upgrade successful").signAsync(fromAccount))], async (result) => {
          expect(result, "Extrinsic is not in the block").to.not.be.undefined;
          const events = result?.[0].events || [];
          expect(
            events.find((evt) => api.events.system.ExtrinsicSuccess.is(evt.event)),
            "Extrinsic is not successful"
          ).toBeTruthy();
        });
      },
    });
  },
});
