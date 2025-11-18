import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { alith, ALITH_ADDRESS, BALTATHAR_ADDRESS, generateKeyringPair } from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { u8aToHex } from "@polkadot/util";
import {
  convertXcmFragmentToVersion,
  descendOriginFromAddress20,
  ERC20_TOTAL_SUPPLY,
  mockHrmpChannelExistanceTx,
  sovereignAccountOfSibling,
  wrapWithXcmVersion,
  XCM_VERSIONS,
  XcmFragment,
} from "../../../../helpers";
import { parseEther } from "ethers";
import type { DispatchError } from "@polkadot/types/interfaces";

describeSuite({
  id: "D024016",
  title: "XCM - DryRunApi",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;

    beforeAll(async function () {
      polkadotJs = context.polkadotJs();
      await context.createBlock();
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: "Should succeed calling DryRunApi::dryRunCall",
        test: async function () {
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Balances")!
            .index.toNumber();

          const randomReceiver =
            "0x1111111111111111111111111111111111111111111111111111111111111111";

          // Beneficiary from destination's point of view
          const destBeneficiary = wrapWithXcmVersion(
            {
              parents: 0,
              interior: {
                X1: {
                  AccountId32: {
                    network: null,
                    id: randomReceiver,
                  },
                },
              },
            },
            xcmVersion
          );

          const assetsToSend = wrapWithXcmVersion(
            [
              {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: Number(balancesPalletIndex) },
                    },
                  },
                },
                fun: {
                  Fungible: 1_000_000_000_000_000n,
                },
              },
            ],
            xcmVersion
          );
          const dest = wrapWithXcmVersion(
            {
              parents: 1,
              interior: {
                Here: null,
              },
            },
            xcmVersion
          );

          const polkadotXcmTx = polkadotJs.tx.polkadotXcm.transferAssets(
            dest,
            destBeneficiary,
            assetsToSend,
            0,
            "Unlimited"
          );

          const dryRunCall = await polkadotJs.call.dryRunApi.dryRunCall(
            { System: { signed: alith.address } },
            polkadotXcmTx,
            xcmVersion
          );

          expect(dryRunCall.isOk).to.be.true;
          expect(dryRunCall.asOk.executionResult.isOk).be.true;
        },
      });

      it({
        id: `T02-XCM-v${xcmVersion}`,
        title: "Should succeed calling DryRunApi::dryRunXcm",
        test: async function () {
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Balances")!
            .index.toNumber();
          const randomKeyPair = generateKeyringPair();

          // We will dry run a "ReserveAssetDeposited" coming from the relay
          let xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: Number(balancesPalletIndex) },
                  },
                },
                fungible: 1_000_000_000_000_000n,
              },
            ],
            beneficiary: u8aToHex(randomKeyPair.addressRaw),
          })
            .reserve_asset_deposited()
            .clear_origin()
            .buy_execution()
            .deposit_asset();

          // Convert to appropriate XCM version
          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

          const dryRunXcm = await polkadotJs.call.dryRunApi.dryRunXcm(
            wrapWithXcmVersion({ parents: 0, interior: { Here: null } }, xcmVersion),
            xcmMessage
          );

          expect(dryRunXcm.isOk).to.be.true;
          expect(dryRunXcm.asOk.executionResult.isComplete).be.true;
        },
      });

      it({
        id: `T03-XCM-v${xcmVersion}`,
        title: "Dry run api should work with erc20 bridget tokens",
        test: async function () {
          const totalErc20Supply = 1_000_000_000_000_000_000n;
          const { contractAddress, status } = await context.deployContract!(
            "ERC20WithInitialSupply",
            {
              args: ["ERC20", "20S", ALITH_ADDRESS, totalErc20Supply],
            }
          );
          expect(status).eq("success");

          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Balances")!
            .index.toNumber();
          const erc20XcmBridgePalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Erc20XcmBridge")!
            .index.toNumber();

          const nativeAmountTransferred = 800_000_000_000_000_000n;
          const erc20AmountTransferred = 3_053_014_345_811_929n;
          const paraId = 2034;
          const paraSovereign = sovereignAccountOfSibling(context, paraId);
          await polkadotJs.tx.balances
            .transferAllowDeath(paraSovereign, nativeAmountTransferred + parseEther("1"))
            .signAndSend(alith);
          const rawTx = await context.writeContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "transfer",
            args: [paraSovereign, erc20AmountTransferred + 1_000_000_000_000n],
            rawTxOnly: true,
          });
          await context.createBlock([rawTx]);

          const origin = wrapWithXcmVersion(
            { parents: 1, interior: { X1: { Parachain: paraId } } },
            xcmVersion
          );
          let xcmMessage = new XcmFragment({
            beneficiary: BALTATHAR_ADDRESS,
            assets: [
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex }, // Balances
                  },
                },
                fungible: nativeAmountTransferred,
              },
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X2: [
                      { PalletInstance: erc20XcmBridgePalletIndex }, // Erc20XcmBridge
                      {
                        AccountKey20: {
                          network: null,
                          key: contractAddress,
                        },
                      },
                    ],
                  },
                },
                fungible: erc20AmountTransferred,
              },
            ],
          })
            .withdraw_asset()
            .clear_origin()
            .buy_execution()
            .deposit_asset()
            .set_topic();

          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

          const dryRunXcm = await polkadotJs.call.dryRunApi.dryRunXcm(origin, xcmMessage);

          expect(dryRunXcm.isOk).to.be.true;
          expect(dryRunXcm.asOk.executionResult.isComplete).be.true;
        },
      });
    }
  },
});
