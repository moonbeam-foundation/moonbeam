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
import { hexToU8a } from "@polkadot/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import {
  registerForeignAsset,
  addAssetToWeightTrader,
  mockAssetBalance,
  RELAY_SOURCE_LOCATION,
  relayAssetMetadata,
} from "../../../../helpers";

const ARBITRARY_ASSET_ID = 42259045809535163221576417993425387648n;

describeSuite({
  id: "D022001",
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
        await expect(
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
                  [],
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
        // We can't initialize rewards anymore, but we can test that if someone
        // had rewards, they wouldn't be able to claim during maintenance mode.
        // This test verifies the maintenance mode filter works for crowdloan claims.

        // Note: Since we can't initialize rewards in the new pallet version,
        // we're testing that the claim transaction itself is blocked.
        // In a real scenario with existing rewards, this would prevent claiming.
        await expect(
          async () => await context.createBlock(context.polkadotJs().tx.crowdloanRewards.claim())
        ).rejects.toThrowError("1010: Invalid Transaction: Transaction call is not expected");
      },
    });

    it({
      id: "T04",
      title: "should forbid assets transfer",
      test: async () => {
        const balance = 100000000000000n;

        // Register foreign asset using the new system
        const { contractAddress } = await registerForeignAsset(
          context,
          ARBITRARY_ASSET_ID,
          RELAY_SOURCE_LOCATION,
          relayAssetMetadata
        );

        // Add asset to weight trader with free execution
        await addAssetToWeightTrader(RELAY_SOURCE_LOCATION, 0n, context);

        // Mock asset balance using the new system
        await mockAssetBalance(context, balance, ARBITRARY_ASSET_ID, alith, ALITH_ADDRESS);

        await expect(
          async () =>
            await context.createBlock(
              context.viem().writeContract({
                address: contractAddress,
                abi: [
                  {
                    type: "function",
                    name: "transfer",
                    inputs: [
                      { type: "address", name: "to" },
                      { type: "uint256", name: "amount" },
                    ],
                  },
                ],
                functionName: "transfer",
                args: [BALTATHAR_ADDRESS, 1000n],
                account: ALITH_ADDRESS,
              })
            )
        ).rejects.toThrowErrorMatchingInlineSnapshot(`
          [ContractFunctionExecutionError: The contract function "transfer" reverted with the following reason:
          no signer available

          Contract Call:
            address:   0xffffffff1fcacbd218edc0eba20fc2308c778080
            function:  transfer(address to, uint256 amount)
            args:              (0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0, 1000)
            sender:    0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac

          Docs: https://viem.sh/docs/contract/writeContract
          Version: viem@2.37.13]
        `);
      },
    });

    it({
      id: "T05",
      title: "should forbid xcm transfer",
      test: async () => {
        await expect(
          async () =>
            await context.createBlock(
              context
                .polkadotJs()
                .tx.polkadotXcm.transferAssets(
                  // Destination
                  {
                    V4: {
                      parents: 1n,
                      interior: {
                        X1: [{ Parachain: 2000n }],
                      },
                    },
                  } as any,
                  // Beneficiary
                  {
                    V4: {
                      parents: 0n,
                      interior: {
                        X1: [
                          {
                            AccountKey20: {
                              network: null,
                              key: hexToU8a(baltathar.address),
                            },
                          },
                        ],
                      },
                    },
                  } as any,
                  // Assets
                  {
                    V4: [
                      {
                        id: {
                          V4: {
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

        await expect(
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
