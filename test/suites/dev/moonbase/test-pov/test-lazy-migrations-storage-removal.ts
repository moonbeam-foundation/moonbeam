import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { alith, baltathar } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { u128 } from "@polkadot/types";

describeSuite({
  id: "D012803",
  title: "Remove LocalAssets storage - PoV Size validation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    let api: ApiPromise;
    beforeAll(async () => {
      api = context.polkadotJs();

      await context.createBlock(api.tx.assets.transfer(assetId, baltathar.address, 1000));
    });

    it({
      id: "T01",
      title: "Validate storage removal uses a reasonable proof size",
      test: async function () {
        const total_entries = 9000;
        // sp_io::hashing::twox_128("LocalAssets".as_bytes());
        const pallet_name_hash = "0xbebaa96ee6c1d0e946832368c6396271";

        const dummy_storage: [string, string][] = [];
        for (let i = 0; i < total_entries; i++) {
          const dummy_data = i.toString(16).padStart(200, "0");
          dummy_storage.push([pallet_name_hash + dummy_data, dummy_data]);
        }

        // Insert 500 entries per block to not reach limits
        for (let i = 0; i < total_entries; i += 500) {
          await context.createBlock(
            api.tx.sudo
              .sudo(api.tx.system.setStorage(dummy_storage.slice(i, i + 500)))
              .signAsync(alith)
          );
        }

        // Check the pallet storage size
        const full_size = (await api.rpc.state.getStorageSize(pallet_name_hash)).toNumber();
        expect(full_size).to.be.equal(1_800_000);

        // editorconfig-checker-disable
        // The constant `MAX_POV_SIZE` comes from: https://github.com/paritytech/polkadot-sdk/blob/b79bf4fb1fec1f7a7483f9a2baa0a1e7a4fcb9c8/polkadot/primitives/src/v6/mod.rs#L391
        // editorconfig-checker-enable
        const MAX_POV_SIZE = 5 * 1024 * 1024; // 5MB
        const reasonable_max_pov_size = MAX_POV_SIZE / 5; // 1MB

        let current_size = full_size;
        while (current_size > 0) {
          // The migration is not complet yet
          expect(
            (await api.query["moonbeamLazyMigrations"].localAssetsMigrationCompleted()).toHuman()
          ).to.be.false;

          // Remove 2000 entries each time
          const entries_to_remove = 2000;
          const result = await context.createBlock(
            api.tx["moonbeamLazyMigrations"]
              .clearLocalAssetsStorage(entries_to_remove)
              .signAsync(alith)
          );

          // Validate that we are within the reasonable proof size
          expect(result.block.proofSize).to.be.lessThan(reasonable_max_pov_size);
          log(
            `Removed ${entries_to_remove} entries from LocalAssets \
            storage (proof_size: ${result.block.proofSize}).`
          );

          // Validate that some storage got removed
          const new_size = (await api.rpc.state.getStorageSize(pallet_name_hash)).toNumber();
          expect(new_size).to.be.lessThan(current_size);

          // Update current storage size
          current_size = new_size;
        }

        // Validate that the whole storage got removed
        current_size = (await api.rpc.state.getStorageSize(pallet_name_hash)).toNumber();
        expect(current_size).to.be.equal(0);

        // The migration should be complete
        expect(
          (await api.query["moonbeamLazyMigrations"].localAssetsMigrationCompleted()).toHuman()
        ).to.be.true;
      },
    });
  },
});
