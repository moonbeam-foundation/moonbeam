import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, execOpenTechCommitteeProposal } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  GLMR,
  alith,
  baltathar,
  createRawTransfer,
} from "@moonwall/util";
import type { PalletAssetsAssetAccount, PalletAssetsAssetDetails } from "@polkadot/types/lookup";
import { hexToU8a } from "@polkadot/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { mockOldAssetBalance } from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;
const RELAYCHAIN_ARBITRARY_ADDRESS_1: string =
  "0x1111111111111111111111111111111111111111111111111111111111111111";
const ARBITRARY_VESTING_PERIOD = 201600n;

describeSuite({
  id: "D012001",
  title: "Maintenance Mode - Filter",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    beforeAll(async () => {
      await execOpenTechCommitteeProposal(
        context,
        context.polkadotJs().tx.maintenanceMode.enterMaintenanceMode()
      );
    });

    it({
      id: "T01",
      title: "should forbid transferring tokens",
      test: async () => {
        await context.createBlock(await createRawTransfer(context, CHARLETH_ADDRESS, 512));
        expect(
          async () =>
            await context.createBlock(
              context.polkadotJs().tx.balances.transferAllowDeath(BALTATHAR_ADDRESS, 1n * GLMR)
            )
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T02",
      title: "should allow EVM extrinsic from sudo",
      test: async () => {
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
        expect(result?.successful).to.be.true;
        expect(await context.viem().getBalance({ address: randomAccount.address })).to.equal(
          100n * GLMR
        );
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

        await mockOldAssetBalance(
          context,
          assetBalance,
          assetDetails,
          alith,
          newAssetId,
          ALITH_ADDRESS
        );

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
      title: "should forbid xcm transfer",
      test: async () => {
        expect(
          async () =>
            await context.createBlock(
              context
                .polkadotJs()
                .tx.polkadotXcm.transferAssets(
                  // Destination
                  {
                    V5: {
                      parents: 1n,
                      interior: {
                        X1: [{ Parachain: 2000n }],
                      },
                    },
                  } as any,
                  // Beneficiary
                  {
                    V5: {
                      parents: 0n,
                      interior: {
                        X1: [{ AccountKey20: { network: null, key: hexToU8a(baltathar.address) } }],
                      },
                    },
                  } as any,
                  // Assets
                  {
                    V5: [
                      {
                        id: {
                          V5: {
                            parents: 0n,
                            interior: {
                              Here: null,
                            },
                          },
                        },
                        fun: { Fungible: 100n * GLMR },
                      },
                    ],
                  },
                  0, // FeeAssetItem
                  {
                    Limited: { refTime: 8000000000, proofSize: 128 * 1024 },
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
                  transactWeights as any,
                  false
                )
                .signAsync(baltathar)
            )
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });
  },
});
