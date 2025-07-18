import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { alith } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import type { u128 } from "@polkadot/types";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { hexToBigInt } from "@polkadot/util";
import { type Abi, encodeFunctionData } from "viem";
import {
  RELAY_SOURCE_LOCATION,
  mockOldAssetBalance,
  registerOldForeignAsset,
  relayAssetMetadata,
  verifyLatestBlockFees,
} from "../../../../helpers/index.js";
import {
  type RawXcmMessage,
  XcmFragment,
  type XcmFragmentConfig,
  descendOriginFromAddress20,
  injectHrmpMessageAndSeal,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D024131",
  title: "Mock XCM - Send EVM transaction through and pay with xcDOT",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let sendingAddress: `0x${string}`;
    let descendAddress: `0x${string}`;
    let api: ApiPromise;
    let assetId: u128;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;

    const initialSenderBalance: bigint = 10_000_000_000_000_000n;

    beforeAll(async () => {
      api = context.polkadotJs();

      // Register DOT as foreign asset, obtaining xcDOTs
      const { registeredAssetId } = await registerOldForeignAsset(
        context,
        RELAY_SOURCE_LOCATION,
        relayAssetMetadata as any,
        1
      );

      // Descend address from origin address
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      descendAddress = descendOriginAddress;

      // Create types for funding descend address
      const balance = api.createType("Balance", initialSenderBalance);
      assetId = api.createType("u128", hexToBigInt(registeredAssetId as `0x${string}`));

      const assetBalance: PalletAssetsAssetAccount = api.createType("PalletAssetsAssetAccount", {
        balance: balance,
      });
      const assetDetails: PalletAssetsAssetDetails = api.createType("PalletAssetsAssetDetails", {
        supply: balance,
      });

      // Fund descend address with enough xcDOTs to pay XCM message and EVM execution fees
      await mockOldAssetBalance(
        context,
        assetBalance,
        assetDetails,
        alith,
        assetId,
        descendAddress
      );

      // Deploy example contract to be called through XCM
      const { contractAddress, abi } = await context.deployContract!("Incrementor");
      contractABI = abi;
      contractDeployed = contractAddress;

      await verifyLatestBlockFees(context);
    });

    it({
      id: "T01",
      title: "should execute EVM remote call through XCM paying fees in DOT",
      test: async function () {
        // Since we cannot infer the actual weight of the inner message,
        // we are using big enough gas limits to be able to execute the whole xcm transaction.
        const xcmTransaction = {
          V1: {
            gas_limit: 155_000,
            fee_payment: {
              Auto: {
                Low: null,
              },
            },
            action: {
              Call: contractDeployed,
            },
            value: 0,
            input: encodeFunctionData({
              abi: contractABI,
              functionName: "incr",
              args: [],
            }),
            access_list: null,
          },
        };

        const XCDOT_FEE_AMOUNT = 100_000n;

        const config: XcmFragmentConfig = {
          assets: [
            {
              // We refer to DOT here, because XCM will internally convert it to xcDOT
              multilocation: {
                parents: 1,
                interior: { Here: null },
              },
              fungible: XCDOT_FEE_AMOUNT,
            },
          ],
          weight_limit: {
            refTime: 120_000_000_000,
            proofSize: 90_583,
          } as any,
          descend_origin: sendingAddress,
          beneficiary: sendingAddress,
        };

        const transferCall = api.tx.ethereumXcm.transact(xcmTransaction);
        const transferCallEncoded = transferCall?.method.toHex();

        // Build XCM transaction with remote EVM call
        const xcmMessage = new XcmFragment(config)
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 50_000_000_000,
                proofSize: 50_000,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v3();

        let senderBalance = (await api.query.assets.account(assetId, descendAddress))
          .unwrap()
          .balance.toBigInt();

        expect(senderBalance).toBe(initialSenderBalance);
        // Send an XCM and create block to execute it
        await injectHrmpMessageAndSeal(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        senderBalance = (await api.query.assets.account(assetId, descendAddress))
          .unwrap()
          .balance.toBigInt();

        // Check that xcDOT where debited from Alith to pay the fees of the XCM execution
        expect(initialSenderBalance - senderBalance).toBe(XCDOT_FEE_AMOUNT);
      },
    });
  },
});
