import "@moonbeam-network/api-augment";
import { alith, beforeAll, describeSuite, expect } from "moonwall";
import { type Abi, encodeFunctionData, parseAbi } from "viem";
import {
  XcmFragment,
  XCM_VERSIONS,
  type RawXcmMessage,
  injectHrmpMessageAndSeal,
  descendOriginFromAddress20,
  type MultiLocation,
  convertXcmFragmentToVersion,
  registerForeignAsset,
  foreignAssetBalance,
  addAssetToWeightTrader,
  mockAssetBalance,
  ConstantStore,
} from "../../../../helpers";

describeSuite({
  id: "D023915",
  title: "Mock XCM - receive horizontal transact ETHEREUM (asset fee)",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    const assetMetadata = {
      name: "FOREIGN",
      symbol: "FOREIGN",
      decimals: 12n,
      isFrozen: false,
    };
    const statemint_para_id = 1001;
    const statemint_assets_pallet_instance = 50;

    const ASSET_MULTILOCATION: MultiLocation = {
      parents: 1,
      interior: {
        X3: [
          { Parachain: statemint_para_id },
          { PalletInstance: statemint_assets_pallet_instance },
          { GeneralIndex: 0n },
        ],
      },
    };

    const STATEMINT_LOCATION = {
      Xcm: ASSET_MULTILOCATION,
    };

    const assetId = 1n;
    let sendingAddress: `0x${string}`;
    let descendedAddress: `0x${string}`;
    let contractDeployed: `0x${string}`;
    let contractABI: Abi;

    const assetsToTransfer = 100_000_000_000_000_000n;

    let STORAGE_READ_COST: bigint;
    let GAS_LIMIT_POV_RATIO: number;

    beforeAll(async () => {
      const specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
      const constants = ConstantStore(context);
      GAS_LIMIT_POV_RATIO = Number(constants.GAS_PER_POV_BYTES.get(specVersion));
      STORAGE_READ_COST = constants.STORAGE_READ_COST;
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      descendedAddress = descendOriginAddress;

      const { contractAddress, abi } = await context.deployContract!("Incrementor");

      contractDeployed = contractAddress;
      contractABI = abi;

      const { contractAddress: assetAddress } = await registerForeignAsset(
        context,
        assetId,
        STATEMINT_LOCATION,
        assetMetadata
      );

      // Expect to confirm the contract has the correct decimals

      expect(
        await context.viem().readContract({
          address: assetAddress as `0x${string}`,
          functionName: "decimals",
          args: [],
          abi: parseAbi(["function decimals() view returns (uint8)"]),
        })
      ).toBe(12);

      // Foreign asset is registered with the same price as the native token
      await addAssetToWeightTrader(STATEMINT_LOCATION, 1_000_000_000_000_000_000n, context);

      // Fund the descended account with enough FOREIGN assets so that it can pay
      // for XCM fees and the two subsequent EVM transactions in this test.
      await mockAssetBalance(context, assetsToTransfer, assetId, alith, descendedAddress);
    });

    for (const xcmVersion of XCM_VERSIONS) {
      it({
        id: `T01-XCM-v${xcmVersion}`,
        title: `should receive transact and should be able to execute (XCM v${xcmVersion})`,
        test: async function () {
          // Ensure the descended account is (re)funded for each XCM version so that
          // previous runs do not deplete the balance for subsequent versions.
          await mockAssetBalance(context, assetsToTransfer, assetId, alith, descendedAddress);

          const initialBalance = await foreignAssetBalance(context, assetId, descendedAddress);
          const initialNonce = await context
            .viem()
            .getTransactionCount({ address: descendedAddress });

          // Sanity check that the descended account has been pre-funded.
          expect(initialBalance).to.be.gte(assetsToTransfer);

          // Get initial contract count
          const initialCount = (
            await context.viem().call({
              to: contractDeployed,
              data: encodeFunctionData({ abi: contractABI, functionName: "count" }),
            })
          ).data;
          const initialCountBigInt = BigInt(initialCount!.toString());
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

            const transferCall = context.polkadotJs().tx.ethereumXcm.transact(xcmTransaction);
            const transferCallEncoded = transferCall?.method.toHex();

            // We are going to test that we can receive a transact operation from parachain 1
            // using descendOrigin first
            let xcmMessage = new XcmFragment({
              assets: [
                {
                  multilocation: ASSET_MULTILOCATION,
                  fungible: assetsToTransfer / 2n,
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
                  // 100_000 gas + 1 db read (41_742_000)
                  requireWeightAtMost: {
                    refTime: 2_525_000_000n + STORAGE_READ_COST,
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

            const actualCalls = (
              await context.viem().call({
                to: contractDeployed,
                data: encodeFunctionData({ abi: contractABI, functionName: "count" }),
              })
            ).data;

            expect(BigInt(actualCalls!.toString()) - initialCountBigInt).to.eq(expectedCalls);
          }
          // Make sure the descended address has consumed some or all of its funds
          // paying for XCM fees and EVM execution.
          const finalBalance = await foreignAssetBalance(context, assetId, descendedAddress);
          expect(finalBalance).to.be.lte(initialBalance);

          const nonce = await context.viem().getTransactionCount({ address: descendedAddress });
          expect(nonce - initialNonce).to.be.eq(2);
        },
      });
    }
  },
});
