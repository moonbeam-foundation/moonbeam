import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import type { KeyringPair } from "@polkadot/keyring/types";
import { type Abi, encodeFunctionData, parseAbi } from "viem";
import { generateKeyringPair } from "@moonwall/util";
import {
  XcmFragment,
  XCM_VERSIONS,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  type MultiLocation,
  weightMessage,
  convertXcmFragmentToVersion,
  registerForeignAsset,
  foreignAssetBalance,
  addAssetToWeightTrader,
  ConstantStore,
} from "../../../../helpers";

describeSuite({
  id: "D024024",
  title: "Mock XCM - receive horizontal transact ETHEREUM (asset fee)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const assetMetadata = {
      name: "FOREIGN",
      symbol: "FOREIGN",
      decimals: 12n,
      isFrozen: false,
    };
    const statemint_para_id = 1001;
    const statemint_assets_pallet_instance = 50;

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

    const assetId = 1n;
    let sendingAddress: `0x${string}`;
    let descendedAddress: `0x${string}`;
    let random: KeyringPair;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;

    const assetsToTransfer = 100_000_000_000_000_000n;

    let STORAGE_READ_COST: bigint;

    let GAS_LIMIT_POV_RATIO: number;

    beforeAll(async () => {
      const specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
      const constants = ConstantStore(context);
      GAS_LIMIT_POV_RATIO = Number(constants.GAS_PER_POV_BYTES.get(specVersion));
      STORAGE_READ_COST = constants.STORAGE_READ_COST;
      const { contractAddress, abi } = await context.deployContract!("Incrementor");

      contractDeployed = contractAddress;
      contractABI = abi;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      descendedAddress = descendOriginAddress;
      random = generateKeyringPair();

      const { contractAddress: assetAddress } = await registerForeignAsset(
        context,
        assetId,
        STATEMINT_LOCATION,
        assetMetadata
      );

      // Expect to confirm the contract has the correct decimals

      expect(
        await context.viem().readContract({
          address: assetAddress as `0x${string}`,
          functionName: "decimals",
          args: [],
          abi: parseAbi(["function decimals() view returns (uint8)"]),
        })
      ).toBe(12);

      // Foreign asset is registered with the same price as the native token
      await addAssetToWeightTrader(STATEMINT_LOCATION, 1_000_000_000_000_000_000n, context);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `should receive transact and should be able to execute (XCM v${xcmVersion})`,
        test: async function () {
          const initialBalance = await foreignAssetBalance(context, assetId, descendedAddress);
          const initialNonce = await context
            .viem()
            .getTransactionCount({ address: descendedAddress });

          const config = {
            assets: [
              {
                multilocation: ASSET_MULTILOCATION,
                fungible: assetsToTransfer,
              },
            ],
            beneficiary: descendedAddress,
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
                  .deposit_asset()
                  .as_v3()
              ) as any
          );

          // Foreign asset was registered with the same price as the native token
          // so we can calculate the fees using the txPaymentApi
          const nativeFees = (await context
            .polkadotJs()
            .call.transactionPaymentApi.queryWeightToFee({
              refTime: chargedWeight,
              proofSize: 0n,
            })) as bigint;
          const feesToAdd = BigInt(nativeFees.toLocaleString()); // If not converted via string, fees seem to overfund the account

          // we modify the config now:
          // we send assetsToTransfer plus whatever we will be charged in weight
          config.assets[0].fungible = assetsToTransfer + feesToAdd;

          // Construct the real message
          const xcmMessage = new XcmFragment(config)
            .reserve_asset_deposited()
            .clear_origin()
            .buy_execution()
            .deposit_asset()
            .as_v3();

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, statemint_para_id, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          // Make sure descended address has the transferred foreign assets (minus the xcm fees).
          const descendedBalance = await foreignAssetBalance(context, assetId, descendedAddress);
          expect(descendedBalance - initialBalance).to.eq(assetsToTransfer);

          // Get initial contract count
          const initialCount = (
            await context.viem().call({
              to: contractDeployed,
              data: encodeFunctionData({ abi: contractABI, functionName: "count" }),
            })
          ).data;
          const initialCountBigInt = BigInt(initialCount!.toString());
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
          for (const xcmTransaction of xcmTransactions) {
            expectedCalls++;

            const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
            const transferCallEncoded = transferCall?.method.toHex();

            // We are going to test that we can receive a transact operation from parachain 1
            // using descendOrigin first
            let xcmMessage = new XcmFragment({
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
              });

            // Convert to appropriate XCM version
            xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

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

            expect(BigInt(actualCalls!.toString()) - initialCountBigInt).to.eq(expectedCalls);
          }
          // Make sure descended address has no funds
          const finalBalance = await foreignAssetBalance(context, assetId, descendedAddress);
          expect(finalBalance).to.eq(0n);

          const nonce = await context.viem().getTransactionCount({ address: descendedAddress });
          expect(nonce - initialNonce).to.be.eq(2);
        },
      });
    }
  },
});
