import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { BN } from "@polkadot/util";
import { DispatchError } from "@polkadot/types/interfaces";

describeSuite({
  id: "D0110",
  title: "XCM - asset manager - register local asset",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should fail to register a local asset",
      test: async function () {
        const parachainOne = context.polkadotJs();
        // registerForeignAsset
        const { result } = await context.createBlock(
          parachainOne.tx.sudo.sudo(
            parachainOne.tx.assetManager.registerLocalAsset(
              ALITH_ADDRESS,
              ALITH_ADDRESS,
              true,
              new BN(1)
            )
          )
        );
        const err = result?.events.find(({ event: { section } }) => section.toString() === "sudo")
          ?.event.data[0] as DispatchError;

        expect(err.type).eq("Err");
      },
    });
  },
});
