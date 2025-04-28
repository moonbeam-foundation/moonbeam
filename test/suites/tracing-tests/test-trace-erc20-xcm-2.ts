import { beforeAll, customDevRpcRequest, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { hexToNumber, parseEther } from "viem";
import {
  ERC20_TOTAL_SUPPLY,
  XcmFragment,
  type XcmFragmentConfig,
  expectEVMResult,
  injectEncodedHrmpMessageAndSeal,
  injectHrmpMessage,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
} from "../../helpers";

describeSuite({
  id: "T21",
  title: "Trace ERC20 xcm #2",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let erc20ContractAddress: string;
    let eventEmitterAddress: `0x${string}`;
    let ethXcmTxHash: string;
    let regularEthTxHash: string;
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

      // Create an XCM  message that try to transfer more than available
      const failedConfig: XcmFragmentConfig = {
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
            fungible: amountTransferred * 2n, // Try to transfer twice the available amount
          },
        ],
        beneficiary: CHARLETH_ADDRESS,
      };

      const failedXcmMessage = new XcmFragment(failedConfig)
        .withdraw_asset()
        .clear_origin()
        .buy_execution()
        .deposit_asset(2n)
        .as_v3();

      // Mock the reception of the failed xcm message N times
      // N should be high enough to fill IdleMaxServiceWeigh
      // The goal is to have at least one XCM message queud for next block on_initialize 
      for (let i = 0; i < 20; i++) {
        await injectHrmpMessage(context, paraId, {
          type: "XcmVersionedXcm",
          payload: failedXcmMessage,
        });
      }
      await context.createBlock();

      // By calling deployContract() a new block will be created,
      // including the ethereum-xcm transaction (on_initialize) + regular ethereum transaction
      const { contractAddress: eventEmitterAddress_ } = await context.deployContract!(
        "EventEmitter",
        {
          from: alith.address,
        } as any
      );
      eventEmitterAddress = eventEmitterAddress_;

      // The old buggy runtime rollback the eth-xcm tx because XCM executor rollback evm reverts
      regularEthTxHash = (await context.viem().getBlock()).transactions[0];

      // Get the latest block events
      const block = await context.polkadotJs().rpc.chain.getBlock();
      const allRecords = await context.polkadotJs().query.system.events.at(block.block.header.hash);

      // Compute XCM message ID
      const messageHash = context.polkadotJs().createType("XcmVersionedXcm", failedXcmMessage).hash;

      // Find messageQueue.Processed event with matching message ID
      const processedEvent = allRecords.find(
        ({ event }) =>
          event.section === "messageQueue" &&
          event.method === "Processed" &&
          event.data[0].toString() === messageHash.toHex()
      );

      expect(processedEvent).to.not.be.undefined;
    });

    // IMPORTANT: this test will fail once we will merge https://github.com/moonbeam-foundation/moonbeam/pull/3258
    it({
      id: "T01",
      title: "should doesn't include the failed ERC20 xcm transaction in block trace",
      test: async function () {
        const number = await context.viem().getBlockNumber();
        const trace = await customDevRpcRequest("debug_traceBlockByNumber", [
          number.toString(),
          { tracer: "callTracer" },
        ]);

        // Verify that only the regular eth transaction is included in the block trace.
        expect(trace.length).to.eq(1);

        // 1st traced transaction is regular ethereum transaction.
        // - `From` is Alith's adddress.
        // - `To` is the ethereum contract address.
        const txHash = trace[0].txHash;
        expect(txHash).to.eq(regularEthTxHash);
        const call = trace[0].result;
        expect(call.from).to.eq(alith.address.toLowerCase());
        expect(call.to).to.eq(eventEmitterAddress.toLowerCase());
        expect(call.type).be.eq("CREATE");
      },
    });
  },
});
