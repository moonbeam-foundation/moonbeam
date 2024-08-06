import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, beforeEach, describeSuite, expect } from "@moonwall/cli";
import { BALTATHAR_ADDRESS, KeyringPair, alith, generateKeyringPair } from "@moonwall/util";
import { bnToHex } from "@polkadot/util";
import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress20,
  expectOk,
  injectHrmpMessage,
} from "../../../../helpers";

// Below should be the calculation:
// export const TARGET_FILL_AMOUNT =
//   ((MAX_BLOCK_WEIGHT * 0.75 * 0.25 - EXTRINSIC_BASE_WEIGHT) / MAX_BLOCK_WEIGHT) * 1_000_000_000;
// In 0.9.43 rootTesting::fillBlock() now uses more weight so we need to account for that
const TARGET_FILL_AMOUNT = 262_212_900;

// Note on the values from 'transactionPayment.nextFeeMultiplier': this storage item is actually a
// FixedU128, which is basically a u128 with an implicit denominator of 10^18. However, this
// denominator is omitted when it is queried through the API, leaving some very large numbers.
//
// To make sense of them, basically remove 18 zeroes (divide by 10^18). This will give you the
// number used internally by transaction-payment for fee calculations.
describeSuite({
  id: "D011604",
  title: "Fee Multiplier - XCM Executions",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const startingBn = 2000000000000000000n;
    let sendingAddress: string;
    let random: KeyringPair;
    let transferredBalance: bigint;
    let balancesPalletIndex: number;

    beforeAll(async function () {
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      random = generateKeyringPair();
      transferredBalance = 100_000_000_000_000_000_000n;

      await expectOk(
        context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(descendOriginAddress, transferredBalance * 100n)
        )
      );

      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() == "Balances")!
        .index.toNumber();
    });

    beforeEach(async () => {
      const MULTIPLIER_STORAGE_KEY = context
        .polkadotJs()
        .query.transactionPayment.nextFeeMultiplier.key(0)
        .toString();

      await context
        .polkadotJs()
        .tx.sudo.sudo(
          context
            .polkadotJs()
            .tx.system.setStorage([
              [MULTIPLIER_STORAGE_KEY, bnToHex(startingBn, { isLe: true, bitLength: 128 })],
            ])
        )
        .signAndSend(alith);
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should decay with no activity",
      test: async function () {
        const initialValue = await context
          .polkadotJs()
          .query.transactionPayment.nextFeeMultiplier();
        await context.createBlock();
        const postValue = await context.polkadotJs().query.transactionPayment.nextFeeMultiplier();
        expect(initialValue.gt(postValue), "Fee Multiplier value not decayed").to.be.true;
      },
    });

    it({
      id: "T02",
      title: "should not decay when block size at target amount",
      test: async function () {
        const initialValue = await context
          .polkadotJs()
          .query.transactionPayment.nextFeeMultiplier();
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(context.polkadotJs().tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
        );

        const postValue = await context.polkadotJs().query.transactionPayment.nextFeeMultiplier();

        // this is useful to manually find out what is the
        // TARGET_FILL_AMOUNT that will result in a static fee multiplier
        // run the tests with
        // pnpm moonwall test dev_moonbase -d test-fees D011604T02
        // console.log(`pre  ${initialValue.toHuman()}`);
        // console.log(`post ${postValue.toHuman()}`);
        // console.log(`diff ${initialValue.sub(postValue)}`);

        expect(initialValue.eq(postValue), "Fee multiplier not static on ideal fill ratio").to.be
          .true;
      },
    });

    it({
      id: "T03",
      title: "should increase when above target fill ratio",
      test: async function () {
        const initialValue = await context
          .polkadotJs()
          .query.transactionPayment.nextFeeMultiplier();

        let nonce = (
          await context.polkadotJs().query.system.account(alith.address)
        ).nonce.toNumber();
        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(BALTATHAR_ADDRESS, 1_000_000_000_000_000_000n)
          .signAndSend(alith, { nonce: nonce++ });
        await context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
          .signAndSend(alith, { nonce: nonce++ });
        await context.createBlock();

        const postValue = await context.polkadotJs().query.transactionPayment.nextFeeMultiplier();
        expect(
          initialValue.lt(postValue),
          "Fee multiplier not increased when above ideal fill ratio"
        ).to.be.true;
      },
    });

    it({
      id: "T04",
      title: "should not increase fees with xcm activity",
      test: async () => {
        const transferCallEncoded = context
          .polkadotJs()
          .tx.balances.transferAllowDeath(random.address, transferredBalance / 10n)
          .method.toHex();

        const initialValue = await context
          .polkadotJs()
          .query.transactionPayment.nextFeeMultiplier();
        const initialBalance = (await context.polkadotJs().query.system.account(random.address))
          .data.free;
        const initialHeight = (
          await context.polkadotJs().rpc.chain.getBlock()
        ).block.header.number.toNumber();

        await context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
          .signAndSend(alith, { nonce: -1 });
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: transferredBalance / 3n,
            },
          ],
          weight_limit: {
            refTime: 9_000_000_000n,
            proofSize: 150_000n,
          },
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 1000000000n,
                proofSize: 80000n,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v3();

        await injectHrmpMessage(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Enqueue XCM message
        await context.createBlock();
        const postValue = await context.polkadotJs().query.transactionPayment.nextFeeMultiplier();
        expect(initialValue.eq(postValue), "Fee Multiplier has changed between blocks").to.be.true;

        // Process xcm message
        await context.createBlock();

        const postBalance = (await context.polkadotJs().query.system.account(random.address)).data
          .free;
        const postHeight = (
          await context.polkadotJs().rpc.chain.getBlock()
        ).block.header.number.toNumber();

        expect(initialHeight).to.equal(postHeight - 2);
        expect(initialBalance.lt(postBalance), "Expected balances not updated").to.be.true;
      },
    });

    it({
      id: "T05",
      title: "should not increase fees with xcm ETH activity",
      test: async () => {
        const amountToTransfer = transferredBalance / 10n;
        const xcmTransactions = [
          {
            V1: {
              gas_limit: 21000,
              fee_payment: {
                Auto: {
                  Low: null,
                },
              },
              action: {
                Call: random.address,
              },
              value: amountToTransfer,
              input: [],
              access_list: null,
            },
          },
          {
            V2: {
              gas_limit: 21000,
              action: {
                Call: random.address,
              },
              value: amountToTransfer,
              input: [],
              access_list: null,
            },
          },
        ];
        const transferCallEncodedV1 = context
          .polkadotJs()
          .tx.ethereumXcm.transact(xcmTransactions[0] as any)
          .method.toHex();
        const transferCallEncodedV2 = context
          .polkadotJs()
          .tx.ethereumXcm.transact(xcmTransactions[1] as any)
          .method.toHex();

        const initialValue = await context
          .polkadotJs()
          .query.transactionPayment.nextFeeMultiplier();
        const initialBalance = (await context.polkadotJs().query.system.account(random.address))
          .data.free;
        const initialHeight = (
          await context.polkadotJs().rpc.chain.getBlock()
        ).block.header.number.toNumber();

        await context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.rootTesting.fillBlock(TARGET_FILL_AMOUNT))
          .signAndSend(alith, { nonce: -1 });
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: transferredBalance / 3n,
            },
          ],
          weight_limit: {
            refTime: 9_000_000_000n,
            proofSize: 150_000n,
          },
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 1000000000n,
                proofSize: 80000n,
              },
              call: {
                encoded: transferCallEncodedV1,
              },
            },
          })
          .as_v3();

        await injectHrmpMessage(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // Enqueue XCM message
        await context.createBlock();
        const postValue = await context.polkadotJs().query.transactionPayment.nextFeeMultiplier();
        expect(initialValue.eq(postValue), "Fee Multiplier has changed between blocks").to.be.true;

        // Process xcm message
        await context.createBlock();

        const postBalance = (await context.polkadotJs().query.system.account(random.address)).data
          .free;
        const postHeight = (
          await context.polkadotJs().rpc.chain.getBlock()
        ).block.header.number.toNumber();
        expect(initialHeight).to.equal(postHeight - 2);
        expect(initialBalance.lt(postBalance), "Expected balances not updated").to.be.true;
      },
    });
  },
});
