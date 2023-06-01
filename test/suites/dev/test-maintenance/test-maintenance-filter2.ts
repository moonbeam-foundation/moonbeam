import "@moonbeam-network/api-augment";
import {
  beforeAll,
  beforeEach,
  describeSuite,
  execTechnicalCommitteeProposal,
  expect,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  GLMR,
  alith,
  baltathar,
  createRawTransfer,
} from "@moonwall/util";
import { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { RELAY_SOURCE_LOCATION, mockAssetBalance } from "../../../helpers/assets.js";
import { BN, hexToU8a } from "@polkadot/util";
import { u128 } from "@polkadot/types-codec";
import { customDevRpcRequest } from "../../../helpers/common.js";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;
const RELAYCHAIN_ARBITRARY_ADDRESS_1: string =
  "0x1111111111111111111111111111111111111111111111111111111111111111";
const ARBITRARY_VESTING_PERIOD = 201600n;

describeSuite({
  id: "D1802",
  title: "Maintenance Mode - Filter2",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;
    const foreignParaId = 2000;
    let foreignAssetId: u128;

    beforeAll(async () => {
      // registering asset
      const balance = context.polkadotJs().createType("Balance", 100000000000000);
      const assetBalance: PalletAssetsAssetAccount = context
        .polkadotJs()
        .createType("PalletAssetsAssetAccount", {
          balance: balance,
        });

      assetId = context.polkadotJs().createType("u128", ARBITRARY_ASSET_ID);
      const assetDetails: PalletAssetsAssetDetails = context
        .polkadotJs()
        .createType("PalletAssetsAssetDetails", {
          supply: balance,
        });

      await mockAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        baltathar.address
      );

      // setAssetUnitsPerSecond
      await context.createBlock(
        context
          .polkadotJs()
          .tx.sudo.sudo(
            context.polkadotJs().tx.assetManager.setAssetUnitsPerSecond(RELAY_SOURCE_LOCATION, 0, 0)
          )
      );

      await execTechnicalCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it({
      id: "T01",
      title: "should queue DMP until resuming operations",
      test: async function () {
        // Send RPC call to inject DMP message
        // You can provide a message, but if you don't a downward transfer is the default
        await customDevRpcRequest("xcm_injectDownwardMessage", [[]]);

        // Create a block in which the XCM should be executed
        await context.createBlock();

        // Make sure the state does not have ALITH's DOT tokens
        const alithBalance = await context
          .polkadotJs()
          .query.assets.account(assetId.toU8a(), ALITH_ADDRESS);

        // Alith balance is 0
        expect(alithBalance.isNone).to.eq(true);

        // turn maintenance off
        await execTechnicalCommitteeProposal(
          context,
          context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
        );

        // Create a block in which the XCM will be executed
        await context.createBlock();

        // Make sure the state has ALITH's to DOT tokens
        const newAlithBalance = (
          await context.polkadotJs().query.assets.account(assetId.toU8a(), ALITH_ADDRESS)
        ).unwrap();

        // Alith balance is 10 DOT
        expect(newAlithBalance.balance.toBigInt()).to.eq(10000000000000n);
      },
    });
  },
});
