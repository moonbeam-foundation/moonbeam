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
  id: "D3421",
  title: "Mock XCM - receive horizontal transact ETHEREUM (call)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;

    beforeAll(async () => {
      const { contractAddress, abi } = await context.deployContract!("Incrementor");
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
      title: "should receive transact and should be able to execute",
      test: async function () {
        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const xcmTransactions = [
          {
            V1: {
              gas_limit: 100000,
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
                functionName: "incr",
                args: [],
              }),
              access_list: null,
            },
          },
          {
            V2: {
              gas_limit: 100000,
              action: {
                Call: contractDeployed,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: contractABI,
                functionName: "incr",
                args: [],
              }),
              access_list: null,
            },
          },
        ];

        let expectedCalls = 0n;

        for (const xcmTransaction of xcmTransactions) {
          expectedCalls++;

          // TODO need to update lookup types for xcm ethereum transaction V2
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
            weight_limit: new BN(4000000000),
            descend_origin: sendingAddress,
          })
            .descend_origin()
            .withdraw_asset()
            .buy_execution()
            .push_any({
              Transact: {
                originType: "SovereignAccount",
                requireWeightAtMost: 3000000000n,
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

          const actualCalls = await context.readContract!({
            contractAddress: contractDeployed,
            contractName: "Incrementor",
            functionName: "count",
          });

          expect(BigInt(actualCalls!.toString())).to.eq(expectedCalls);
        }
      },
    });
  },
});
