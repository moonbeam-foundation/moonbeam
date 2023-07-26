import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { Web3 } from "web3";
import { ApiPromise } from "@polkadot/api";
import {
  XcmFragment,
  XcmFragmentConfig,
  injectHrmpMessage,
  sovereignAccountOfSibling,
} from "../../../helpers/xcm.js";
import { parseEther } from "ethers";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D2704",
  title: "Mock XCM - Send local erc20",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let erc20ContractAddress: string;
    let w3: Web3;
    let polkadotJs: ApiPromise;

    beforeEach(async function () {
      w3 = context.web3();
      polkadotJs = context.polkadotJs();

      const { contractAddress, status } = await context.deployContract!("ERC20WithInitialSupply", {
        args: ["ERC20", "20S", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
      });
      erc20ContractAddress = contractAddress;
      expect(status).eq("success");
    });

    it({
      id: "T01",
      title: "Should be able to transfer ERC20 token through xcm with xtokens precomp",
      test: async function () {
        const amountTransferred = 10n;

        // Destination as multilocation
        const destination = [
          // one parent
          1,
          // This represents X1(AccountKey20(BALTATHAR_ADDRESS, NetworkAny))
          // AccountKey20 variant (03) + the 20 bytes account + Any network variant (00)
          ["0x03" + BALTATHAR_ADDRESS.slice(2) + "00"],
        ];

        const balanceBefore = (
          await polkadotJs.query.system.account(ALITH_ADDRESS)
        ).data.free.toBigInt();

        const rawTx = await context.writePrecompile!({
          precompileName: "Xtokens",
          functionName: "transfer",
          args: [
            // address of the multiasset
            erc20ContractAddress,
            // amount
            amountTransferred,
            // Destination as multilocation
            destination,
            // weight
            500_000n,
          ],
          gas: 500_000n,
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        const balanceAfter = (
          await polkadotJs.query.system.account(ALITH_ADDRESS)
        ).data.free.toBigInt();

        const receipt = await w3.eth.getTransactionReceipt(result!.hash);
        const gasPrice = receipt.effectiveGasPrice;
        const fees = BigInt(receipt.gasUsed) * BigInt(gasPrice!);

        // Fees should have been spent
        expect(balanceAfter).to.equal(balanceBefore - fees);

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).equals(ERC20_TOTAL_SUPPLY - amountTransferred);
      },
    });

    it({
      id: "T02",
      title: "Mock XCM - Receive back erc20",
      test: async function () {
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferred = 1_000n;

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
              fungible: 100_000_000_000_000_000n,
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
          .deposit_asset(2n)
          .as_v2();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });
        await context.createBlock();

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(amountTransferred);
      },
    });
  },
});
