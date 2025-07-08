import "@moonbeam-network/api-augment";
import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";

import type { KeyringPair } from "@polkadot/keyring/types";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  type RawXcmMessage,
  sovereignAccountOfSibling,
  injectHrmpMessage,
} from "../../../../helpers/xcm.js";

const foreign_para_id = 2000;

describeSuite({
  id: "D024101",
  title: "Auto-pause XCM",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let transferredBalance: bigint;
    let sovereignAddress: string;
    let random: KeyringPair;
    let balancesPalletIndex: number;

    beforeAll(async () => {
      random = generateKeyringPair();
      sovereignAddress = sovereignAccountOfSibling(context, 2000);
      transferredBalance = 1_000_000_000_000_000n;

      await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(sovereignAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(sovereignAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);

      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();
    });

    it({
      id: "T01",
      title: "Should automatically pause xcm when block production is stuck",
      test: async function () {
        await context.createBlock();

        // XCM Mode should be equal to Normal
        expect((await context.polkadotJs().query.emergencyParaXcm.mode()).isNormal).to.be.true;

        // Create a dummy xcm message to test auto-pause
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: transferredBalance,
            },
          ],
          beneficiary: random.address,
        })
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v5();

        // Inject an XCM message that should be included in the next block but not executed
        await injectHrmpMessage(context, foreign_para_id, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Simulate block production stall (skip more than PausedThreshold relay blocks)
        await customDevRpcRequest("test_skipRelayBlocks", [301]);

        // Create a new block, this block should pause XCM incoming execution
        await context.createBlock([], {
          expectEvents: [context.polkadotJs().events.emergencyParaXcm.EnteredPausedXcmMode],
          allowFailures: false,
        });

        // XCM Mode should be equal to Paused
        expect((await context.polkadotJs().query.emergencyParaXcm.mode()).isPaused).to.be.true;

        // Produce some blocks when XCm is Paused
        await context.createBlock();
        await context.createBlock();

        // The sovereign account of foreign parachain sould still have funds
        const balance = (
          await context.polkadotJs().query.system.account(sovereignAddress)
        ).data.free.toBigInt();
        expect(balance, "Sovereign account balance has changed").to.eq(transferredBalance);

        // The beneficiary of the XCM message should not have funds
        const randomBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();
        expect(randomBalance, "beneficiary of the XCM message receive funds").to.eq(0n);

        // Sudo should be able to resume XCM execution
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.emergencyParaXcm.pausedToNormal()),
          {
            expectEvents: [context.polkadotJs().events.emergencyParaXcm.NormalXcmOperationResumed],
            allowFailures: false,
          }
        );

        // XCM Mode should be equal to Normal
        expect((await context.polkadotJs().query.emergencyParaXcm.mode()).isNormal).to.be.true;

        // The next block should execute previous XCM message
        await context.createBlock([], {
          expectEvents: [],
          allowFailures: false,
        });

        // The sovereign account of foreign parachain should now be empty
        const balance2 = (
          await context.polkadotJs().query.system.account(sovereignAddress)
        ).data.free.toBigInt();
        expect(balance2, "Sovereign account not empty, transfer has failed").to.eq(0n);

        // The beneficiary of the XCM message should now have funds
        const randomBalance2 = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();
        expect(randomBalance2, "beneficiary balance not increased, transfer has failed").to.not.eq(
          0n
        );
      },
    });
  },
});
