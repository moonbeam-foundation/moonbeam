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
  id: "D2701",
  title: "Mock XCM - Test bad contract with excess gas usage",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;
    let w3: Web3;
    const paraId = 888;

    beforeEach(async function () {
      polkadotJs = context.polkadotJs();
      w3 = context.web3();
    });

    it({
      id: "T01",
      title: "Should be able to transfer ERC20 token through xcm with xtokens precomp",
      test: async function () {
        const amountTransferred = 1_000n;

        // Destination as multilocation
        const destination = [
          // one parent
          1,
          // This represents X1(AccountKey20(BALTATHAR_ADDRESS, NetworkAny))
          // AccountKey20 variant (03) + the 20 bytes account + Any network variant (00)
          ["0x03" + BALTATHAR_ADDRESS.slice(2) + "00"],
        ];

        // Deploy contract
        const { contractAddress, status } = await context.deployContract!("ERC20ExcessGas", {
          args: ["ERC20", "20S", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
        });
        expect(status).eq("success");

        const balanceBefore = (
          await polkadotJs.query.system.account(ALITH_ADDRESS)
        ).data.free.toBigInt();

        const rawTx = await context.writePrecompile!({
          precompileName: "Xtokens",
          functionName: "transfer",
          args: [
            // address of the multiasset
            contractAddress,
            // amount
            amountTransferred,
            // Destination as multilocation
            destination,
            // weight
            4_000_000n,
          ],
          gas: 200_000n,
          rawTxOnly: true,
        });

        // Tx should have failed
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Revert");

        const balanceAfter = (
          await polkadotJs.query.system.account(ALITH_ADDRESS)
        ).data.free.toBigInt();

        const receipt = await w3.eth.getTransactionReceipt(result!.hash);
        const gasPrice = receipt.effectiveGasPrice;
        const fees = BigInt(receipt.gasUsed) * BigInt(gasPrice!);

        // Fees should have been spent
        expect(balanceAfter).to.equal(balanceBefore - fees);

        // ERC20 balance shouldn't change
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).equals(ERC20_TOTAL_SUPPLY);
      },
    });

    it({
      id: "T02",
      title: "Mock XCM - Receive back erc20",
      test: async function () {
        const amountTransferred = 1_000n;
        const paraSovereign = sovereignAccountOfSibling(context, 888);

        // Deploy contract
        const { contractAddress, status } = await context.deployContract!("ERC20ExcessGas", {
          args: ["ERC20", "20S", paraSovereign, ERC20_TOTAL_SUPPLY],
        });
        expect(status).eq("success");

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
                        key: contractAddress,
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
          .deposit_asset_v3(2n)
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });
        await context.createBlock();

        // Charleth shouldn't have received any tokens
        expect(
          await context.readContract!({
            contractName: "ERC20ExcessGas",
            contractAddress: contractAddress as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(0n);
      },
    });
  },
});
