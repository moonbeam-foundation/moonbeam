import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { GAS_LIMIT_POV_RATIO } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import {
  XcmFragment,
  RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
} from "../../../../helpers/xcm.js";

describeSuite({
  id: "D014021",
  title: "Mock XCM - transact ETHEREUM input size check succeeds",
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
        context
          .polkadotJs()
          .tx.balances.transferAllowDeath(descendOriginAddress, transferredBalance),
        { allowFailures: false }
      );

      const balance = (
        await context.polkadotJs().query.system.account(descendOriginAddress)
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);
    });

    it({
      id: "T01",
      title: "should succeed to call the contract",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        // Matches the BoundedVec limit in the runtime.
        const CALL_INPUT_SIZE_LIMIT = Math.pow(2, 16);

        const GAS_LIMIT = 1000000;

        const xcmTransactions = [
          {
            V1: {
              gas_limit: GAS_LIMIT,
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
                    .utils.bytesToHex(new Uint8Array(CALL_INPUT_SIZE_LIMIT - 128).fill(0)),
                ],
              }),
              access_list: null,
            },
          },
          {
            V2: {
              gas_limit: GAS_LIMIT,
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
                    .utils.bytesToHex(new Uint8Array(CALL_INPUT_SIZE_LIMIT - 128).fill(0)),
                ],
              }),
              access_list: null,
            },
          },
        ];

        for (const xcmTransaction of xcmTransactions) {
          const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
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
            weight_limit: {
              refTime: 40000000000,
              proofSize: (GAS_LIMIT / GAS_LIMIT_POV_RATIO) * 2,
            } as any,
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originKind: "SovereignAccount",
                requireWeightAtMost: {
                  refTime: 30000000000,
                  proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
                },
                call: {
                  encoded: transferCallEncoded,
                },
              },
            })
            .as_v3();

          // Send an XCM and create block to execute it
          await injectHrmpMessageAndSeal(context, 1, {
            type: "XcmVersionedXcm",
            payload: xcmMessage,
          } as RawXcmMessage);

          const block = await context.viem().getBlock({ blockTag: "latest" });
          // Input size is valid - on the limit -, expect block to include a transaction.
          // That means the pallet-ethereum-xcm decoded the provided input to a BoundedVec.
          expect(block.transactions.length).to.be.eq(1);
        }
      },
    });
  },
});
