import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { parseEther } from "ethers";
import { expectEVMResult } from "../../../../helpers";
import {
  XcmFragment,
  XcmFragmentConfig,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";

export const ERC20_TOTAL_SUPPLY = 1_000_000_000n;

describeSuite({
  id: "D014035",
  title: "Mock XCM V3 - XCM Weight Limit",
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
      title: "Check that MaxAssetsIntoHolding limit is enforced",
      test: async function () {
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferred = 1_000_000n;

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
              fungible: 1_700_000_000_000_000n,
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

        // first check with n=limit-1 and check n=limit increases weight
        const getTransferWeight = async function (limit: bigint) {
          // Mock the reception of the xcm message
          await injectHrmpMessageAndSeal(context, paraId, {
            type: "XcmVersionedXcm",
            payload: new XcmFragment(config)
              .withdraw_asset()
              .clear_origin()
              .buy_execution()
              .deposit_asset_v3(limit)
              .as_v3(),
          });

          const allRecords = await polkadotJs.query.system.events();
          const [{ weightUsed }] = allRecords
            .filter(({ event }) => polkadotJs.events.messageQueue.Processed.is(event))
            .map((e) => e.event.data as unknown as { weightUsed: { proofSize: unknown } });
          return Number(weightUsed.proofSize);
        };

        const limit = 64n;
        // get weight for n=limit-1 and n=limit
        const weight_under = await getTransferWeight(limit - 1n);
        const weight_limit = await getTransferWeight(limit);

        // assert that n=limit-1 increases weight
        expect(weight_under).lt(weight_limit);

        // now check that n=limit+1 does not increase weight
        let weight_over = await getTransferWeight(limit + 1n);
        expect(weight_over).eq(weight_limit);

        // check abusive n>>>limit does not increase weight
        weight_over = await getTransferWeight(BigInt(1e9));
        expect(weight_over).eq(weight_limit);
      },
    });
  },
});
