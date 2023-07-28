import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import {
  XcmFragment,
  XcmFragmentConfig,
  injectHrmpMessage,
  sovereignAccountOfSibling,
} from "../../../helpers/xcm.js";
import { SignedBlock } from "@polkadot/types/interfaces";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D2710",
  title: "Mock XCM V3 - Receive erc20 via XCM",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
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
      title: "Should be able to transfer ERC20 token through incoming XCM message",
      test: async function () {
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferred = 1_000_000n;

        // Get pallet indices
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
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferred);

        // Create the incoming xcm message
        const config: XcmFragmentConfig = {
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

        const xcmMessage = new XcmFragment(config)
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3(100n)
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });
        const {
          block: { hash: blockHash },
        } = await context.createBlock();

        const block = await polkadotJs.rpc.chain.getBlock(blockHash);

        // console.log(blockEvents.block.extrinsics[1].toHuman());

        // no blockHash is specified, so we retrieve the latest
        // const signedBlock = await api.rpc.chain.getBlock();

        // get the api and events at a specific block
        const apiAt = await polkadotJs.at(blockHash);
        const allRecords = await apiAt.query.system.events();

        // allRecords.forEach(({ event }) => {
        //   console.log(event.toHuman());
        // });

        const a = allRecords.filter(
          ({ event: { section, method } }) =>
            section === "xcmpQueue" && method === "OverweightEnqueued"
        );

        expect(a.length).eq(1);

        // Erc20 tokens should have been received
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
  },
});
