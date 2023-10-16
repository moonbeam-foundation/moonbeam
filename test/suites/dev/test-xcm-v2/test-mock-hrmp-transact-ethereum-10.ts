import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { BN } from "@polkadot/util";
import { Abi, encodeFunctionData } from "viem";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../helpers/xcm.js";

describeSuite({
  id: "D3429",
  title: "Mock XCM - transact ETHEREUM input size check fails",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("CallForwarder");
      contractDeployed = contractAddress;
      contractABI = abi;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      transferredBalance = 10_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovreign account
      await context.createBlock(
        context.polkadotJs().tx.balances.transfer(descendOriginAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(descendOriginAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "should fail to call the contract due to BoundedVec restriction",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        // Matches the BoundedVec limit in the runtime.
        const CALL_INPUT_SIZE_LIMIT = Math.pow(2, 16);

        const xcmTransactions = [
          {
            V1: {
              gas_limit: 1000000,
              fee_payment: {
                Auto: {
                  Low: null,
                },
              },
              action: {
                Call: contractDeployed,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: contractABI,
                functionName: "call",
                args: [
                  "0x0000000000000000000000000000000000000001",
                  context
                    .web3()
                    .utils.bytesToHex(new Uint8Array(CALL_INPUT_SIZE_LIMIT - 127).fill(0)),
                ],
              }),
              access_list: null,
            },
          },
          {
            V2: {
              gas_limit: 1000000,
              action: {
                Call: contractDeployed,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: contractABI,
                functionName: "call",
                args: [
                  "0x0000000000000000000000000000000000000001",
                  context
                    .web3()
                    .utils.bytesToHex(new Uint8Array(CALL_INPUT_SIZE_LIMIT - 127).fill(0)),
                ],
              }),
              access_list: null,
            },
          },
        ];

        for (const xcmTransaction of xcmTransactions) {
          const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction as any);
          const transferCallEncoded = transferCall?.method.toHex();
          // We are going to test that we can receive a transact operation from parachain 1
          // using descendOrigin first
          const xcmMessage = new XcmFragment({
            assets: [
              {
                multilocation: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: balancesPalletIndex },
                  },
                },
                fungible: transferredBalance / 2n,
              },
            ],
            weight_limit: new BN(40000000000),
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originType: "SovereignAccount",
                requireWeightAtMost: 30000000000n,
                call: {
                  encoded: transferCallEncoded,
                },
              },
            })
            .as_v2();

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, 1, {
            type: "StagingXcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          const block = await context.viem().getBlock({ blockTag: "latest" });
          // Input size is invalid by 1 byte, expect block to not include a transaction.
          // That means the pallet-ethereum-xcm couldn't decode the provided input to a BoundedVec.
          expect(block.transactions.length).to.be.eq(0);
        }
      },
    });
  },
});
