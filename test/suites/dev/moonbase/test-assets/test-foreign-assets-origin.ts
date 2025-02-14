import "@moonbeam-network/api-augment";
import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import {
  ARBITRARY_ASSET_ID,
  RELAY_SOURCE_LOCATION_V4,
  registerForeignAsset,
  relayAssetMetadata,
  sendCallAsPara,
  sovereignAccountOfSibling,
} from "../../../../helpers";
import { parseAbi } from "viem";
import { fundAccount } from "../../../../helpers/balances";

describeSuite({
  id: "D010110",
  title: "XCM - Origin Tests",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const calls = [
      context.polkadotJs().tx.evmForeignAssets.createForeignAsset(
        1n,
        {
          parents: 1,
          interior: {
            X3: [{ Parachain: 3000 }, { PalletInstance: 5 }, { GeneralIndex: 5 }],
          },
        },
        18n,
        "TEST",
        "TEST"
      ),
      context.polkadotJs().tx.evmForeignAssets.changeXcmLocation(1n, {
        parents: 1,
        interior: {
          X3: [{ Parachain: 3000 }, { PalletInstance: 6 }, { GeneralIndex: 6 }],
        },
      }),
      context.polkadotJs().tx.evmForeignAssets.freezeForeignAsset(1n, false),
      context.polkadotJs().tx.evmForeignAssets.unfreezeForeignAsset(1n),
    ];

    it({
      id: "T01",
      title: "Cannot call externsics using normal account",
      test: async function () {
        for (const call of calls) {
          const { result } = await context.createBlock(call);
          expect(result.error?.name).to.be.eq("BadOrigin");
        }
      },
    });

    it({
      id: "T02",
      title: "Cannot call externsics using sovereign account",
      test: async function () {
        const fundAmount = 100_000_000_000_000_000_000_000n;
        const siblingParaSovereignAccounts = sovereignAccountOfSibling(context, 3000);
        await fundAccount(siblingParaSovereignAccounts as `0x${string}`, fundAmount, context);

        for (const call of calls) {
          const { errorName } = await sendCallAsPara(call, 3000, context, fundAmount / 20n, true, {
            originKind: "SovereignAccount",
          });
          expect(errorName).to.be.eq("BadOrigin");
        }
      },
    });
  },
});
