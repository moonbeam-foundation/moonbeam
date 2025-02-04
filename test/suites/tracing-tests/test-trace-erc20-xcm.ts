import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { hexToNumber, parseEther } from "viem";
import {
  ERC20_TOTAL_SUPPLY,
  XcmFragment,
  type XcmFragmentConfig,
  expectEVMResult,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../helpers";

describeSuite({
  id: "T09",
  title: "Trace ERC20 xcm",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let erc20ContractAddress: string;
    let transactionHash: string;

    beforeAll(async () => {
      const { contractAddress, status } = await context.deployContract!("ERC20WithInitialSupply", {
        args: ["ERC20", "20S", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY],
      });
      erc20ContractAddress = contractAddress;
      expect(status).eq("success");

      const paraId = 888;
      const paraSovereign = sovereignAccountOfSibling(context, paraId);
      const amountTransferred = 1_000_000n;

      // Get pallet indices
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      const balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();
      const erc20XcmPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Erc20XcmBridge")!
        .index.toNumber();

      // Send some native tokens to the sovereign account of paraId (to pay fees)
      await context
        .polkadotJs()
        .tx.balances.transferAllowDeath(paraSovereign, parseEther("1"))
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

      const xcmMessage = new XcmFragment(config)
        .withdraw_asset()
        .clear_origin()
        .buy_execution()
        .deposit_asset(2n)
        .as_v3();

      // Mock the reception of the xcm message
      await injectHrmpMessageAndSeal(context, paraId, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      });

      transactionHash = (await context.viem().getBlock()).transactions[0];

      // Erc20 tokens should have been received
      expect(
        await context.readContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress as `0x${string}`,
          functionName: "balanceOf",
          args: [CHARLETH_ADDRESS],
        })
      ).equals(amountTransferred);
    });

    it({
      id: "T01",
      title: "should trace ERC20 xcm transaction with debug_traceTransaction",
      test: async function () {
        const receipt = await context
          .viem()
          .getTransactionReceipt({ hash: transactionHash as `0x${string}` });
        const trace = await customDevRpcRequest("debug_traceTransaction", [
          transactionHash,
          { tracer: "callTracer" },
        ]);
        // We traced the transaction, and the traced gas used should be greater* than or equal
        // to the one recorded in the ethereum transaction receipt.
        // *gasUsed on tracing does not take into account gas refund.
        expect(hexToNumber(trace.gasUsed)).gte(Number(receipt.gasUsed));
      },
    });
  },
});
