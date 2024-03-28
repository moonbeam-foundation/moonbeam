import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import { expectEVMResult, getTransactionFees } from "../../../../helpers";
import {
  XcmFragment,
  XcmFragmentConfig,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D014026",
  title: "Mock XCM - Send two local ERC20",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let erc20ContractAddress1: string;
    let erc20ContractAddress2: string;

    let polkadotJs: ApiPromise;

    beforeEach(async function () {
      polkadotJs = context.polkadotJs();

      // Deploy first contract
      const contract1 = await context.deployContract!("ERC20WithInitialSupply", {
        args: ["First", "FIR", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
      });
      erc20ContractAddress1 = contract1.contractAddress;
      expect(contract1.status).eq("success");

      // Deploy second contract
      const contract2 = await context.deployContract!("ERC20WithInitialSupply", {
        args: ["Second", "SEC", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
      });
      erc20ContractAddress2 = contract2.contractAddress;
      expect(contract2.status).eq("success");
    });

    it({
      id: "T01",
      title: "Should be able to transfer two ERC20 tokens through xtokens precompile",
      test: async function () {
        const amountTransferred = 1000n;

        const balanceBefore = (
          await polkadotJs.query.system.account(ALITH_ADDRESS)
        ).data.free.toBigInt();

        // Destination as multilocation
        const destination = [
          // one parent
          1,
          // This represents X1(AccountKey20(BALTATHAR_ADDRESS, NetworkAny))
          // AccountKey20 variant (03) + the 20 bytes account + Any network variant (00)
          ["0x03" + BALTATHAR_ADDRESS.slice(2) + "00"],
        ];

        const currency1 = [erc20ContractAddress1, amountTransferred];
        const currency2 = [erc20ContractAddress2, amountTransferred];

        const rawTx = await context.writePrecompile!({
          precompileName: "Xtokens",
          functionName: "transferMultiCurrencies",
          args: [
            // address of the multiassets
            [currency1, currency2],
            // index fee
            1n,
            // Destination as multilocation
            destination,
            // weight
            500_000_000n,
          ],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        const fees = await getTransactionFees(context, result!.hash);

        const balanceAfter = (
          await polkadotJs.query.system.account(ALITH_ADDRESS)
        ).data.free.toBigInt();

        // Fees should have been spent
        expect(balanceAfter).to.equal(balanceBefore - fees);

        // Erc20 tokens of the first contract should have been spent
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress1 as `0x${string}`,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).equals(ERC20_TOTAL_SUPPLY - amountTransferred);

        // Erc20 tokens of the second contract should have been spent
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress2 as `0x${string}`,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).equals(ERC20_TOTAL_SUPPLY - amountTransferred);
      },
    });
    it({
      id: "T02",
      title: "Should not be able to transfer two ERC20 through incoming XCM message",
      test: async function () {
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferredOf1 = 1_000_000n;
        const amountTransferredOf2 = 2_000_000n;

        // Get pallet indices
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

        // Send some erc20 tokens (of first contract) to the sovereign account of paraId
        const rawTx = await context.writeContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress1 as `0x${string}`,
          functionName: "transfer",
          args: [paraSovereign, amountTransferredOf1],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        // Check the sovereign account has received ERC20 tokens (of first contract)
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress1 as `0x${string}`,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferredOf1);

        // Send some ERC20 tokens (of second contract) to the sovereign account of paraId
        const rawTx2 = await context.writeContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress2 as `0x${string}`,
          functionName: "transfer",
          args: [paraSovereign, amountTransferredOf2],
          rawTxOnly: true,
        });

        const { result: result2 } = await context.createBlock(rawTx2);
        expectEVMResult(result2!.events, "Succeed");

        // Check the sovereign account has received ERC20 tokens (of second contract)
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress2 as `0x${string}`,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferredOf2);

        // Create the xcm message to send ERC20s to Charleth
        const config: XcmFragmentConfig = {
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
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
                        network: null,
                        key: erc20ContractAddress2,
                      },
                    },
                  ],
                },
              },
              fungible: amountTransferredOf2,
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
                        key: erc20ContractAddress1,
                      },
                    },
                  ],
                },
              },
              fungible: amountTransferredOf1,
            },
          ],
          beneficiary: CHARLETH_ADDRESS,
        };

        // Build the xcm message
        const xcmMessage = new XcmFragment(config)
          .withdraw_asset()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3(3n)
          .as_v4();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        // Erc20 tokens (of first contract) should have been received in Charleth's address
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress1 as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(0n);

        // Erc20 tokens (of second contract) should have been received in Charleth's address
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress2 as `0x${string}`,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).equals(0n);
      },
    });
  },
});
