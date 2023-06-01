import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, execTechnicalCommitteeProposal, expect } from "@moonwall/cli";
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
import { hexToU8a } from "@polkadot/util";
import { u128 } from "@polkadot/types-codec";
import { customDevRpcRequest } from "../../../helpers/common.js";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;
const ARBITRARY_ASSET_ID_2 = 37857590458095351632257641799342538748n;
const RELAYCHAIN_ARBITRARY_ADDRESS_1: string =
  "0x1111111111111111111111111111111111111111111111111111111111111111";
const ARBITRARY_VESTING_PERIOD = 201600n;

describeSuite({
  id: "D1801",
  title: "Maintenance Mode - Filter",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let assetId: u128;

    beforeAll(async () => {
      await execTechnicalCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it({
      id: "T01",
      title: "should forbid transferring tokens",
      test: async function () {
        await context.createBlock(await createRawTransfer(context, CHARLETH_ADDRESS, 512));
        expect(
          async () =>
            await context.createBlock(
              context.polkadotJs().tx.balances.transfer(BALTATHAR_ADDRESS, 1n * GLMR)
            )
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T02",
      title: "should allow EVM extrinsic from sudo",
      test: async function () {
        const randomAccount = privateKeyToAccount(generatePrivateKey());
        const { result } = await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.evm.call(
                  ALITH_ADDRESS,
                  randomAccount.address,
                  "0x0",
                  100n * GLMR,
                  12_000_000n,
                  10_000_000_000n,
                  "0",
                  null,
                  []
                )
            )
        );
        expect(result!.successful).to.be.true;
        expect(
          await context.viemClient("public").getBalance({ address: randomAccount.address })
        ).to.equal(100n * GLMR);
      },
    });

    it({
      id: "T03",
      title: "should forbid crowdloan rewards claim",
      test: async () => {
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.initializeRewardVec([
                  [RELAYCHAIN_ARBITRARY_ADDRESS_1, CHARLETH_ADDRESS, 3_000_000n * GLMR],
                ])
            )
        );
        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        await context.createBlock(
          context
            .polkadotJs()
            .tx.sudo.sudo(
              context
                .polkadotJs()
                .tx.crowdloanRewards.completeInitialization(
                  initBlock.toBigInt() + ARBITRARY_VESTING_PERIOD
                )
            )
        );

        expect(
          async () => await context.createBlock(context.polkadotJs().tx.crowdloanRewards.claim())
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T04",
      title: "should forbid assets transfer",
      test: async () => {
        const balance = context.polkadotJs().createType("Balance", 100000000000000);
        const assetBalance: PalletAssetsAssetAccount = context
          .polkadotJs()
          .createType("PalletAssetsAssetAccount", {
            balance: balance,
          });

        const newAssetId = context.polkadotJs().createType("u128", ARBITRARY_ASSET_ID);
        const assetDetails: PalletAssetsAssetDetails = context
          .polkadotJs()
          .createType("PalletAssetsAssetDetails", {
            supply: balance,
          });

        await mockAssetBalance(context, assetBalance, assetDetails, alith, newAssetId, alith.address);

        expect(
          async () =>
            await context.createBlock(
              context.polkadotJs().tx.assets.transfer(newAssetId, BALTATHAR_ADDRESS, 1000)
            )
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T05",
      title: "should forbid xtokens transfer",
      test: async () => {
        expect(
          async () =>
            await context.createBlock(
              context
                .polkadotJs()
                .tx.xTokens.transfer(
                  "SelfReserve", //enum
                  100n * GLMR,
                  {
                    V3: {
                      parents: 1n,
                      interior: {
                        X2: [
                          { Parachain: 2000n },
                          { AccountKey20: { network: null, key: hexToU8a(baltathar.address) } },
                        ],
                      },
                    },
                  } as any,
                  {
                    Limited: { refTime: 4000000000, proofSize: 64 * 1024 },
                  }
                )
                .signAsync(baltathar)
            )
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T06",
      title: "should forbid xcmTransactor to",
      test: async () => {
        const transactWeights = context
          .polkadotJs()
          .createType("PalletXcmTransactorTransactWeights", {
            transactRequiredWeightAtMost: 0,
            overallWeight: null,
          });

        const fee = context.polkadotJs().createType("PalletXcmTransactorCurrencyPayment", {
          currency: {
            AsCurrencyId: {
              SelfReserve: null,
            },
          },
          feeAmount: null,
        });

        expect(
          async () =>
            await context.createBlock(
              context
                .polkadotJs()
                .tx.xcmTransactor.transactThroughDerivative(
                  "Relay",
                  0,
                  fee as any,
                  "",
                  transactWeights as any
                )
                .signAsync(baltathar)
            )
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

  },
});

//   it("should queue DMP until resuming operations", async function () {
//     // Send RPC call to inject DMP message
//     // You can provide a message, but if you don't a downward transfer is the default
//     await customWeb3Request(context.web3, "xcm_injectDownwardMessage", [[]]);

//     // Create a block in which the XCM should be executed
//     await context.createBlock();

//     // Make sure the state does not have ALITH's DOT tokens
//     let alithBalance = await context.polkadotJs().query.assets.account(
//       assetId.toU8a(),
//       alith.address
//     );

//     // Alith balance is 0
//     expect(alithBalance.isNone).to.eq(true);

//     // turn maintenance off
//     await execTechnicalCommitteeProposal(
//       context,
//       context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
//     );

//     // Create a block in which the XCM will be executed
//     await context.createBlock();

//     // Make sure the state has ALITH's to DOT tokens
//     const newAlithBalance = (
//       await context.polkadotJs().query.assets.account(assetId.toU8a(), alith.address)
//     ).unwrap();

//     // Alith balance is 10 DOT
//     expect(newAlithBalance.balance.toBigInt()).to.eq(10000000000000n);
//   });
// });

// describeDevMoonbeam("Maintenance Mode - Filter", (context) => {
//   let assetId: string;
//   const foreignParaId = 2000;

//   before("registering asset", async function () {
//     const assetMetadata = {
//       name: "FOREIGN",
//       symbol: "FOREIGN",
//       decimals: new BN(12),
//       isFroze: false,
//     };

//     const sourceLocation = {
//       Xcm: { parents: 1, interior: { X1: { Parachain: foreignParaId } } },
//     };

//     // registerForeignAsset
//     const {
//       result: { events: eventsRegister },
//     } = await context.createBlock(
//       context.polkadotJs().tx.sudo.sudo(
//         context.polkadotJs().tx.assetManager.registerForeignAsset(
//           sourceLocation,
//           assetMetadata,
//           new BN(1),
//           true
//         )
//       )
//     );

//     assetId = eventsRegister
//       .find(({ event: { section } }) => section.toString() === "assetManager")
//       .event.data[0].toHex()
//       .replace(/,/g, "");

//     // setAssetUnitsPerSecond
//     await context.createBlock(
//       context.polkadotJs().tx.sudo.sudo(
//         context.polkadotJs().tx.assetManager.setAssetUnitsPerSecond(sourceLocation, 0, 0)
//       )
//     );
//   });

//   before("entering maintenant mode", async () => {
//     await execTechnicalCommitteeProposal(
//       context,
//       context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
//     );
//   });

//   it("should queue XCM messages until resuming operations", async function () {
//     // Send RPC call to inject XCMP message
//     // You can provide a message, but if you don't a downward transfer is the default
//     await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [foreignParaId, []]);

//     // Create a block in which the XCM should be executed
//     await context.createBlock();

//     // Make sure the state does not have ALITH's foreign asset tokens
//     let alithBalance = (await context.polkadotJs().query.assets.account(
//       assetId,
//       alith.address
//     )) as any;
//     // Alith balance is 0
//     expect(alithBalance.isNone).to.eq(true);

//     // turn maintenance off
//     await execTechnicalCommitteeProposal(
//       context,
//       context.polkadotJs().tx.maintenanceMode.resumeNormalOperation()
//     );

//     // Create a block in which the XCM will be executed
//     await context.createBlock();

//     // Make sure the state has ALITH's to foreign assets tokens
//     alithBalance = (
//       await context.polkadotJs().query.assets.account(assetId, alith.address)
//     ).unwrap();

//     expect(alithBalance.balance.toBigInt()).to.eq(10000000000000n);
//   });
// });
