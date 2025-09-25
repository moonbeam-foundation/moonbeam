import {beforeAll, describeSuite, expect} from "@moonwall/cli";
import {alith, ALITH_ADDRESS, BALTATHAR_ADDRESS, generateKeyringPair} from "@moonwall/util";
import type {ApiPromise} from "@polkadot/api";
import {u8aToHex} from "@polkadot/util";
import {
  convertXcmFragmentToVersion, ERC20_TOTAL_SUPPLY,
  wrapWithXcmVersion,
  XCM_VERSIONS,
  XcmFragment,
} from "../../../../helpers";

describeSuite({
  id: "D024016",
  title: "XCM - DryRunApi",
  foundationMethods: "dev",
  testCases: ({context, it}) => {
    let polkadotJs: ApiPromise;

    beforeAll(async function () {
      polkadotJs = context.polkadotJs();
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: "Should succeed calling DryRunApi::dryRunCall",
        test: async function () {
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({name}) => name.toString() === "Balances")!
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
                      X1: {PalletInstance: Number(balancesPalletIndex)},
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
            {System: {signed: alith.address}},
            polkadotXcmTx,
            xcmVersion
          );

          expect(dryRunCall.isOk).to.be.true;
          expect(dryRunCall.asOk.executionResult.isOk).be.true;
        },
      });

      it({
        id: "T02",
        title: "Should succeed calling DryRunApi::dryRunXcm",
        test: async function () {
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({name}) => name.toString() === "Balances")!
            .index.toNumber();
          const randomKeyPair = generateKeyringPair();

          // We will dry run a "ReserveAssetDeposited" coming from the relay
          let xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X1: {PalletInstance: Number(balancesPalletIndex)},
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
            wrapWithXcmVersion(
              {
                Concrete: {parent: 1, interior: {Here: null}},
              },
              xcmVersion
            ),
            xcmMessage
          );

          expect(dryRunXcm.isOk).to.be.true;
          expect(dryRunXcm.asOk.executionResult.isComplete).be.true;
        },
      });

      it({
        id: "T03",
        title: "Dry run api should work with erc20 bridget tokens",
        test: async function () {
          const {
            contractAddress,
            status
          } = await context.deployContract!("ERC20WithInitialSupply", {
            args: ["ERC20", "20S", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
          });
          expect(status).eq("success");

          const origin = wrapWithXcmVersion(
            {
              Concrete: {parent: 1, interior: {X1: {Parachain: 2034}}},
            },
            xcmVersion
          );
          let xcmMessage = new XcmFragment({
            beneficiary: BALTATHAR_ADDRESS,
            assets: [
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X1: {PalletInstance: 10}, // Balances
                  },
                },
                fungible: 800_000_000_000_000_000n,
              },
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X2: [
                      {PalletInstance: 110}, // Erc20XcmBridge
                      {
                        AccountKey20: {
                          network: null,
                          key: contractAddress,
                        },
                      },
                    ],
                  },
                },
                fungible: 3_053_014_345_811_929n,
              },
            ],
          })
            .withdraw_asset()
            .clear_origin()
            .buy_execution()
            .deposit_asset()
            .set_topic();

          xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

          const dryRunXcm = await polkadotJs.call.dryRunApi.dryRunXcm(
            origin,
            xcmMessage,
          );

          expect(dryRunXcm.isOk).to.be.true;
          expect(dryRunXcm.asOk.executionResult.isComplete).be.true;
        },
      });
    }
  },
});
