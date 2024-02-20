import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";

describeSuite({
  id: "D2406",
  title: `Verify author filter consistency`,
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
    });

    it({
      id: "T01",
      title: `should have eligibility > 0`,
      test: async function () {
        // sp_io::hashing::twox_128("EVM".as_bytes());
        const pallet_name_hash =
          "0xf2794c22e353e9a839f12faab03a911be470c6afbbbc027eb288ade7595953c2";

        const size = await paraApi.rpc.state.getStorageSize(pallet_name_hash);

        let keys: any[] = [];
        let biggest_key = 0;
        for (let i = 0; i < 9000; i += 1000) {
          const res: string[] = (
            await paraApi.rpc.state.getKeysPaged(pallet_name_hash, 1000, keys[keys.length - 1])
          ).toHuman() as any;

          const biggest_key_size =
            res.reduce((c, n) => (c > n.length - 2 ? c : n.length - 2), 0) / 2;
          if (biggest_key_size > biggest_key) {
            biggest_key = biggest_key_size;
          }

          keys = [...keys, ...res];
        }

        console.log(keys.length);

        let biggest_value = 0;
        for (let i = 0; i < keys.length; i += 100) {
          const k = keys.slice(i, i + 100);

          const res = (await paraApi.rpc.state.queryStorageAt(k)) as any;

          const biggest_value_size =
            (res ?? []).reduce((c, n) => {
              try {
                const v = (n?.toHuman()?.length || 2) - 2;
                return c > v ? c : v;
              } catch {
                return c;
              }
            }, 0) / 2;
          if (biggest_value_size > biggest_value) {
            biggest_value = biggest_value_size;
          }
          console.log("Biggest value", biggest_value);
          await new Promise((r) => setTimeout(r, 1000));
        }
        console.log(
          `Pallet storage size: ${size} bytes\nTotal keys: ${keys.length}\nBiggest key: ${biggest_key} bytes\nBiggest value: ${biggest_value} bytes`
        );
      },
    });
  },
});
