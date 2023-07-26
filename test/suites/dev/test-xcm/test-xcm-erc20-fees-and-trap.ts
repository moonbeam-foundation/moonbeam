import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import {
  XcmFragment,
  XcmFragmentConfig,
  injectHrmpMessage,
  sovereignAccountOfSibling,
  weightMessage,
} from "../../../helpers/xcm.js";
import { ALITH_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { stringToU8a } from "@polkadot/util";
import { parseEther } from "ethers";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D2702",
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
        const erc20XcmPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
          (pallet) => pallet.name === "Erc20XcmBridge"
        ).index;

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
                        network: "Any",
                        key: stringToU8a(erc20ContractAddress),
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
          .deposit_asset(2n)
          .as_v2();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });
        await context.createBlock();

        // Search for Fail event
        const records = await polkadotJs.query.system.events();
        const events = records.filter(
          ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
        );

        // Check the error is TooExpensive
        expect(events).to.have.lengthOf(1);
        expect((events[0].event.data.toHuman() as any).error).equals("TooExpensive");

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
        const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
          (pallet) => pallet.name === "Balances"
        ).index;
        const erc20XcmPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
          (pallet) => pallet.name === "Erc20XcmBridge"
        ).index;

        // Send some native tokens to the sovereign account of paraId (to pay fees)
        await polkadotJs.tx.balances.transfer(paraSovereign, parseEther("1")).signAndSend(alith);
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

        // Check the sovereign account has reveived erc20 tokens
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
                        network: "Any",
                        key: stringToU8a(erc20ContractAddress),
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
          .as_v2();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });
        await context.createBlock();

        const chargedWeight = await weightMessage(
          context,
          polkadotJs.createType("XcmVersionedXcm", xcmMessage) as any
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
          .deposit_asset()
          .as_v2();

        const balanceBefore = (
          await polkadotJs.query.system.account(paraSovereign)
        ).data.free.toBigInt();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessageToClaimAssets,
        });
        await context.createBlock();

        // Search for AssetsClaimed event
        const records = await polkadotJs.query.system.events();
        const events = records.filter(
          ({ event }) => event.section == "polkadotXcm" && event.method == "AssetsClaimed"
        );
        expect(events).to.have.lengthOf(1);

        const chargedWeightForClaim = await weightMessage(
          context,
          polkadotJs.createType("XcmVersionedXcm", xcmMessageToClaimAssets) as any
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
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });
        await context.createBlock();

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
                        network: "Any",
                        key: stringToU8a(erc20ContractAddress),
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
          .deposit_asset()
          .as_v2();

        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessageFailedClaim,
        });
        await context.createBlock();

        // Search for UnknownClaim error
        const records2 = await polkadotJs.query.system.events();
        const events2 = records2.filter(
          ({ event }) => event.section == "xcmpQueue" && event.method == "Fail"
        );
        expect(events2).to.have.lengthOf(1);
        expect((events2[0].event.data.toHuman() as any).error).equals("UnknownClaim");

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
