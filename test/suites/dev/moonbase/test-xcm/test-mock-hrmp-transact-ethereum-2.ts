import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";

import { type Abi, encodeFunctionData } from "viem";
import {
  XcmFragment,
  XCM_VERSIONS,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  convertXcmFragmentToVersion,
} from "../../../../helpers/xcm.js";
import { ConstantStore } from "../../../../helpers";

describeSuite({
  id: "D024023",
  title: "Mock XCM - receive horizontal transact ETHEREUM (call)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance: bigint;
    let sendingAddress: `0x${string}`;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;
    let GAS_LIMIT_POV_RATIO: number;

    beforeAll(async () => {
      const specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
      const constants = ConstantStore(context);
      GAS_LIMIT_POV_RATIO = Number(constants.GAS_PER_POV_BYTES.get(specVersion));

      const { contractAddress, abi } = await context.deployContract!("Incrementor");
      contractDeployed = contractAddress;
      contractABI = abi;

      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      transferredBalance = 1_000_000_000_000_000_000_000n;

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

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `should receive transact and should be able to execute (XCM v${xcmVersion})`,
        test: async function () {
          // Get initial contract count
          const initialCount = await context.readContract!({
            contractAddress: contractDeployed,
            contractName: "Incrementor",
            functionName: "count",
          });
          const initialCountBigInt = BigInt(initialCount!.toString());

          // Get Pallet balances index
          const metadata = await context.polkadotJs().rpc.state.getMetadata();
          const balancesPalletIndex = metadata.asLatest.pallets
            .find(({ name }) => name.toString() === "Balances")!
            .index.toNumber();

          const GAS_LIMIT = 100_000;

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
                  functionName: "incr",
                  args: [],
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
            let xcmMessage = new XcmFragment({
              assets: [
                {
                  multilocation: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
                    },
                  },
                  fungible: 1_000_000_000_000_000_000n,
                },
              ],
              descend_origin: sendingAddress,
            })
              .descend_origin()
              .withdraw_asset()
              .buy_execution()
              .push_any({
                Transact: {
                  originKind: "SovereignAccount",
                  requireWeightAtMost: {
                    refTime: 3_000_000_000,
                    proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
                  },
                  call: {
                    encoded: transferCallEncoded,
                  },
                },
              });

            // Convert to appropriate XCM version
            xcmMessage = convertXcmFragmentToVersion(xcmMessage, xcmVersion);

            // Send an XCM and create block to execute it
            await injectHrmpMessageAndSeal(context, 1, {
              type: "XcmVersionedXcm",
              payload: xcmMessage,
            } as RawXcmMessage);

            const actualCalls = await context.readContract!({
              contractAddress: contractDeployed,
              contractName: "Incrementor",
              functionName: "count",
            });

            expect(BigInt(actualCalls!.toString()) - initialCountBigInt).to.eq(expectedCalls);
          }
        },
      });
    }
  },
});
