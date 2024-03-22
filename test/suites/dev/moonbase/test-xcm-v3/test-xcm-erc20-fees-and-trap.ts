import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import { expectEVMResult } from "../../../../helpers";
import {
  XcmFragment,
  XcmFragmentConfig,
  expectXcmEventMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  weightMessage,
} from "../../../../helpers/xcm.js";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D014134",
  title: "Mock XCM - Fails trying to pay fees with ERC20",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let erc20ContractAddress: string;
    let polkadotJs: ApiPromise;

    beforeEach(async function () {
      polkadotJs = context.polkadotJs();

      const { contractAddress, status } = await context.deployContract!("ERC20WithInitialSupply", {
        args: ["ERC20", "20S", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
      });
      erc20ContractAddress = contractAddress;
      expect(status).eq("success");
    });

    it({
      id: "T01",
      title: "Should fail as tries to pay fees with ERC20",
      test: async function () {
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferred = 1_000_000n;

        // Get pallet index
        const metadata = await polkadotJs.rpc.state.getMetadata();
        const erc20XcmPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Erc20XcmBridge")!
          .index.toNumber();

        // Send some erc20 tokens to the sovereign account of paraId
        const rawTx = await context.writeContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress as `0x${string}`,
          functionName: "transfer",
          args: [paraSovereign, amountTransferred],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        // Check the sovereign account has received erc20 tokens
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferred);

        // Create xcm message to send ERC20 tokens to Charleth
        // We don't buy any execution with native currency
        const config: XcmFragmentConfig = {
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X2: [
                    {
                      PalletInstance: erc20XcmPalletIndex,
                    },
                    {
                      AccountKey20: {
                        network: null,
                        key: erc20ContractAddress,
                      },
                    },
                  ],
                },
              },
              fungible: amountTransferred,
            },
          ],
          beneficiary: CHARLETH_ADDRESS,
        };

        // Build the xcm message
        const xcmMessage = new XcmFragment(config)
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3(2n)
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        expect(await expectXcmEventMessage(context, "TooExpensive")).toBe(true);

        // Charleth should not receive ERC20 tokens due to failed execution
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(0n);
      },
    });

    it({
      id: "T02",
      title: "Should not trap any ERC20",
      test: async function () {
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferred = 1_000_000n;

        // Get pallet index
        const metadata = await polkadotJs.rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();
        const erc20XcmPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Erc20XcmBridge")!
          .index.toNumber();

        // Send some native tokens to the sovereign account of paraId (to pay fees)
        await polkadotJs.tx.balances
          .transferAllowDeath(paraSovereign, parseEther("1"))
          .signAndSend(alith);
        await context.createBlock();

        // Send some erc20 tokens to the sovereign account of paraId
        const rawTx = await context.writeContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress as `0x${string}`,
          functionName: "transfer",
          args: [paraSovereign, amountTransferred],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        // Check the sovereign account has received erc20 tokens
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferred);

        const feeAssetAmount = 1_000_000_000_000_000n;

        // Create xcm message to send ERC20 tokens to Charleth
        const config: XcmFragmentConfig = {
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: feeAssetAmount,
            },
            {
              multilocation: {
                parents: 0,
                interior: {
                  X2: [
                    {
                      PalletInstance: erc20XcmPalletIndex,
                    },
                    {
                      AccountKey20: {
                        network: null,
                        key: erc20ContractAddress,
                      },
                    },
                  ],
                },
              },
              fungible: amountTransferred,
            },
          ],
          beneficiary: CHARLETH_ADDRESS,
        };

        // Build the xcm message without deposit_asset()
        // This is to trap all the assets present in the holding register
        const xcmMessage = new XcmFragment(config)
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        const chargedWeight = await weightMessage(
          context,
          polkadotJs.createType("XcmVersionedXcm", xcmMessage)
        );
        // We are charging chargedWeight
        // chargedWeight * 50000 = chargedFee
        const chargedFee = chargedWeight * 50000n;

        const amountOfTrappedAssets = feeAssetAmount - chargedFee;
        const claimConfig = {
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: amountOfTrappedAssets,
            },
          ],
          beneficiary: paraSovereign,
        };
        // Check non-erc20 can be claimed
        const xcmMessageToClaimAssets = new XcmFragment(claimConfig)
          .claim_asset()
          .buy_execution()
          .deposit_asset_v3()
          .as_v3();

        const balanceBefore = (
          await polkadotJs.query.system.account(paraSovereign)
        ).data.free.toBigInt();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessageToClaimAssets,
        });

        // Search for AssetsClaimed event
        const records = await polkadotJs.query.system.events();
        const events = records.filter(
          ({ event }) => event.section == "polkadotXcm" && event.method == "AssetsClaimed"
        );
        expect(events).to.have.lengthOf(1);

        const chargedWeightForClaim = await weightMessage(
          context,
          polkadotJs.createType("XcmVersionedXcm", xcmMessageToClaimAssets)
        );
        // We are charging chargedWeightForClaim
        // chargedWeightForClaim * 50000 = chargedFeeForClaim
        const chargedFeeForClaim = chargedWeightForClaim * 50000n;

        const balanceAfter = (
          await polkadotJs.query.system.account(paraSovereign)
        ).data.free.toBigInt();

        // Check the balance is correct
        expect(balanceAfter).to.equal(balanceBefore + (amountOfTrappedAssets - chargedFeeForClaim));

        // Mock again the reception of the initial xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        const failedClaimConfig: XcmFragmentConfig = {
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X2: [
                    {
                      PalletInstance: erc20XcmPalletIndex,
                    },
                    {
                      AccountKey20: {
                        network: null,
                        key: erc20ContractAddress,
                      },
                    },
                  ],
                },
              },
              fungible: amountTransferred,
            },
          ],
          beneficiary: paraSovereign,
        };

        // Check erc20 cannot be claimed
        const xcmMessageFailedClaim = new XcmFragment(failedClaimConfig)
          .claim_asset()
          .buy_execution()
          .deposit_asset_v3()
          .as_v3();

        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessageFailedClaim,
        });

        expect(await expectXcmEventMessage(context, "UnknownClaim")).toBe(true);

        // Check the sovereign account has the same initial amount of ERC20 tokens
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferred);
      },
    });
  },
});
