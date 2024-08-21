import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import { KeyringPair } from "@polkadot/keyring/types";
import { Abi, encodeFunctionData } from "viem";
import { generateKeyringPair, GAS_LIMIT_POV_RATIO } from "@moonwall/util";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  MultiLocation,
  weightMessage,
} from "../../../../helpers/xcm.js";
import { registerOldForeignAsset } from "../../../../helpers/assets.js";

describeSuite({
  id: "D014025",
  title: "Mock XCM - receive horizontal transact ETHEREUM (asset fee)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    const assetMetadata = {
      name: "FOREIGN",
      symbol: "FOREIGN",
      decimals: new BN(12),
      isFrozen: false,
    };
    const statemint_para_id = 1001;
    const statemint_assets_pallet_instance = 50;
    const palletId = "0x6D6f646c617373746d6E67720000000000000000";

    const ASSET_MULTILOCATION: MultiLocation = {
      parents: 1,
      interior: {
        X3: [
          { Parachain: statemint_para_id },
          { PalletInstance: statemint_assets_pallet_instance },
          { GeneralIndex: 0n },
        ],
      },
    };

    const STATEMINT_LOCATION = {
      Xcm: ASSET_MULTILOCATION,
    };

    let assetId: string;
    let sendingAddress: `0x${string}`;
    let descendedAddress: `0x${string}`;
    let random: KeyringPair;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;

    const assetsToTransfer = 100_000_000_000n;

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("Incrementor");

      contractDeployed = contractAddress;
      contractABI = abi;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      descendedAddress = descendOriginAddress;
      random = generateKeyringPair();

      // registerOldForeignAsset
      const { registeredAssetId, registeredAsset } = await registerOldForeignAsset(
        context,
        STATEMINT_LOCATION,
        assetMetadata,
        1_000_000_000_000
      );
      assetId = registeredAssetId;
      expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

      const config = {
        assets: [
          {
            multilocation: ASSET_MULTILOCATION,
            fungible: assetsToTransfer,
          },
        ],
        beneficiary: descendOriginAddress,
      };

      // How much will the message weight?
      const chargedWeight = await weightMessage(
        context,
        context
          .polkadotJs()
          .createType(
            "XcmVersionedXcm",
            new XcmFragment(config)
              .reserve_asset_deposited()
              .clear_origin()
              .buy_execution()
              .deposit_asset_v3()
              .as_v3()
          ) as any
      );

      // we modify the config now:
      // we send assetsToTransfer plus whatever we will be charged in weight
      config.assets[0].fungible = assetsToTransfer + chargedWeight;

      // Construct the real message
      const xcmMessage = new XcmFragment(config)
        .reserve_asset_deposited()
        .clear_origin()
        .buy_execution()
        .deposit_asset_v3()
        .as_v3();

      // Send an XCM and create block to execute it
      await injectHrmpMessageAndSeal(context, statemint_para_id, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      // Make sure descended address has the transferred foreign assets (minus the xcm fees).
      expect(
        (await context.polkadotJs().query.assets.account(assetId, descendedAddress))
          .unwrap()
          .balance.toBigInt()
      ).to.eq(assetsToTransfer);
    });

    it({
      id: "T01",
      title: "should receive transact and should be able to execute",
      test: async function () {
        const GAS_LIMIT = 100_000;
        const xcmTransactions = [
          {
            V1: {
              gas_limit: GAS_LIMIT,
              fee_payment: {
                Auto: {
                  Low: null,
                },
              },
              action: {
                Call: contractDeployed,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: contractABI,
                functionName: "incr",
                args: [],
              }),
              access_list: null,
            },
          },
          {
            V2: {
              gas_limit: GAS_LIMIT,
              action: {
                Call: contractDeployed,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: contractABI,
                functionName: "incr",
                args: [],
              }),
              access_list: null,
            },
          },
        ];

        let expectedCalls = 0n;
        // TODO: move this to the constant file
        const STORAGE_READ_COST = 41_742_000n;

        for (const xcmTransaction of xcmTransactions) {
          expectedCalls++;

          // TODO need to update lookup types for xcm ethereum transaction V2
          const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
          const transferCallEncoded = transferCall?.method.toHex();

          // We are going to test that we can receive a transact operation from parachain 1
          // using descendOrigin first
          const xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: ASSET_MULTILOCATION,
                fungible: assetsToTransfer / 2n,
              },
            ],
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                // 100_000 gas + 1 db read (41_742_000)
                requireWeightAtMost: {
                  refTime: 2_525_000_000n + STORAGE_READ_COST,
                  proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
                },
                call: {
                  encoded: transferCallEncoded,
                },
              },
            })
            .as_v3();

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, 1, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          const actualCalls = (
            await context.viem().call({
              to: contractDeployed,
              data: encodeFunctionData({ abi: contractABI, functionName: "count" }),
            })
          ).data;

          expect(BigInt(actualCalls!.toString())).to.eq(expectedCalls);
        }
        // Make sure descended address went below existential deposit and was killed
        expect((await context.polkadotJs().query.assets.account(assetId, descendedAddress)).isNone)
          .to.be.true;
        // Even if the account does not exist in assets aymore, we still have a nonce 1. Reason is:
        // - First transact withdrew 1/2 of assets, nonce was increased to 1.
        // - Second transact withdrew the last 1/2 of assets, account was reaped and zeroed.
        // - The subsequent evm execution increased the nonce to 1, even without sufficient
        //   references.
        // We can expect this to be the behaviour on any xcm fragment that completely drains an
        // account to transact ethereum-xcm after.
        const nonce = await context.viem().getTransactionCount({ address: descendedAddress });
        expect(nonce).to.be.eq(1);
      },
    });
  },
});
