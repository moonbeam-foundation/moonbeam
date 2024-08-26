import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, generateKeyringPair } from "@moonwall/util";
import { XcmVersionedXcm } from "@polkadot/types/lookup";
import { u8aToHex } from "@polkadot/util";
import { expectEVMResult, descendOriginFromAddress20, ConstantStore } from "../../../../helpers";

export const CLEAR_ORIGIN_WEIGHT = 5_194_000n;

describeSuite({
  id: "D012801",
  title: "Precompiles - xcm utils",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let STORAGE_READ_COST;
    beforeAll(async function () {
      STORAGE_READ_COST = ConstantStore(context).STORAGE_READ_COST;
    });
    it({
      id: "T01",
      title: "allows to retrieve parent-based ML account",
      test: async function () {
        const multilocation: [number, any[]] = [1, []];
        const expectedAddress = u8aToHex(new Uint8Array([...new TextEncoder().encode("Parent")]))
          .padEnd(42, "0")
          .toLowerCase();

        expect(
          (
            (await context.readPrecompile!({
              precompileName: "XcmUtils",
              functionName: "multilocationToAddress",
              args: [multilocation],
            })) as any
          ).toLowerCase()
        ).to.equal(expectedAddress);
      },
    });

    it({
      id: "T02",
      title: "allows to retrieve parachain-based ML account",
      test: async function () {
        const x2_parachain_asset_enum_selector = "0x00";
        const x2_parachain_id = "000007D0";
        const paraId = context.polkadotJs().createType("ParaId", 2000);

        const multilocation: [number, any[]] = [
          1,
          // Parachain(2000)
          [x2_parachain_asset_enum_selector + x2_parachain_id],
        ];

        const expectedAddress = u8aToHex(
          new Uint8Array([...new TextEncoder().encode("sibl"), ...paraId.toU8a()])
        ).padEnd(42, "0");

        expect(
          (
            (await context.readPrecompile!({
              precompileName: "XcmUtils",
              functionName: "multilocationToAddress",
              args: [multilocation],
            })) as any
          ).toLowerCase()
        ).to.equal(expectedAddress);
      },
    });

    it({
      id: "T03",
      title: "allows to retrieve generic ML-based derivated account",
      test: async function () {
        const x2_parachain_asset_enum_selector = "0x00";
        const x2_parachain_id = "00000001";

        // Junction::AccountKey20
        const account20EnumSelector = "0x03";
        // [0x01; 20]
        const account20Address = "0101010101010101010101010101010101010101";
        // NetworkId::Any
        const account20NetworkId = "00";

        const multilocation: [number, any[]] =
          // Destination as multilocation
          [
            // one parent
            1,
            // X2(Parachain(2000), AccountId32(account32Address))
            [
              x2_parachain_asset_enum_selector + x2_parachain_id,
              account20EnumSelector + account20Address + account20NetworkId,
            ],
          ];

        const { descendOriginAddress } = descendOriginFromAddress20(context);
        expect(
          (
            (await context.readPrecompile!({
              precompileName: "XcmUtils",
              functionName: "multilocationToAddress",
              args: [multilocation],
            })) as any
          ).toLowerCase()
        ).toBe(descendOriginAddress);
      },
    });

    it({
      id: "T04",
      title: "allows to retrieve weight of message",
      test: async function () {
        const message = {
          V2: [
            {
              ClearOrigin: null,
            },
          ],
        };
        const xcm = context.polkadotJs().createType("VersionedXcm", message);

        expect(
          await context.readPrecompile!({
            precompileName: "XcmUtils",
            functionName: "weightMessage",
            args: [xcm.toHex()],
          })
        ).to.equal(CLEAR_ORIGIN_WEIGHT);
      },
    });

    it({
      id: "T05",
      title: "allows to retrieve units per second for an asset",
      test: async function () {
        // Junction::PalletInstance(3)
        const x2_pallet_instance_enum_selector = "0x04";
        const x2_instance = "03";

        // This represents X1(PalletInstance(3)))

        // This multilocation represents our native token
        const asset = [
          // zero parents
          0,
          // X1(PalletInstance)
          // PalletInstance: Selector (04) + palconst instance 1 byte (03)
          [x2_pallet_instance_enum_selector + x2_instance],
        ];

        const expectedUnitsPerSecond = 50_000n * 1_000_000_000_000n;

        expect(
          await context.readPrecompile!({
            precompileName: "XcmUtils",
            functionName: "getUnitsPerSecond",
            args: [asset],
          })
        ).to.equal(expectedUnitsPerSecond);
      },
    });

    it({
      id: "T06",
      title: "allows to execute a custom xcm message",
      test: async function () {
        const random = generateKeyringPair();

        const transferCall = context
          .polkadotJs()
          .tx.balances.transferAllowDeath(random.address, 1n * GLMR);
        const transferCallEncoded = transferCall?.method.toHex();

        const xcmMessage = {
          V2: [
            {
              Transact: {
                originType: "SovereignAccount",
                requireWeightAtMost: 525_000_000n + STORAGE_READ_COST, // 21_000 gas limit
                call: {
                  encoded: transferCallEncoded,
                },
              },
            },
          ],
        };

        const receivedMessage: XcmVersionedXcm = context
          .polkadotJs()
          .createType("XcmVersionedXcm", xcmMessage) as any;

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmExecute",
          args: [receivedMessage.toHex(), 2_000_000_000n],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const testAccountBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();

        expect(testAccountBalance).to.eq(1n * GLMR);
      },
    });

    it({
      id: "T07",
      title: "allows to execute a custom xcm evm to evm, but reentrancy forbids",
      test: async function () {
        const random = generateKeyringPair();

        const ethTx = {
          V1: {
            gas_limit: 21000,
            fee_payment: {
              Auto: {
                Low: null,
              },
            },
            action: {
              Call: random.address,
            },
            value: 1n * GLMR,
            input: [],
            access_list: null,
          },
        };
        const transferCall = context.polkadotJs().tx.ethereumXcm.transact(ethTx as any);
        const transferCallEncoded = transferCall?.method.toHex();

        const xcmMessage = {
          V2: [
            {
              Transact: {
                originType: "SovereignAccount",
                requireWeightAtMost: 525_000_000n + STORAGE_READ_COST, // 21_000 gas limit
                call: {
                  encoded: transferCallEncoded,
                },
              },
            },
          ],
        };

        const receivedMessage: XcmVersionedXcm = context
          .polkadotJs()
          .createType("XcmVersionedXcm", xcmMessage);

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmExecute",
          args: [receivedMessage.toHex(), 4_000_000_000],
          rawTxOnly: true,
          gas: 5_000_000n,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        // Tokens transferred
        const testAccountBalance = (
          await context.polkadotJs().query.system.account(random.address)
        ).data.free.toBigInt();

        expect(testAccountBalance, "Transfer went through, possible EVM re-entrancy").to.eq(0n);
      },
    });

    it({
      id: "T08",
      title: "does not allow to self-send a custom xcm message",
      test: async function () {
        const ownParaId = (await context.polkadotJs().query.parachainInfo.parachainId()) as any;
        const x1_parachain_asset_enum_selector = "0x00";
        const x1_parachain_id = ownParaId.toHex().slice(2);

        // Sending it here
        // { parents:0, Here}
        const destHere: [number, any[]] = [
          // one parents
          0,
          // Here
          [],
        ];

        // Sending it with the representation of the para as seen by the relay
        // { parents:0, parachain(0)}
        const destParaRelayView: [number, any[]] = [
          // one parents
          0,
          // Parachain(0)
          [x1_parachain_asset_enum_selector + x1_parachain_id],
        ];

        // Sending it with the representation of the para as seen by other paras
        // { parents:1, parachain(0)}
        const destParaOtherParaView: [number, any[]] = [
          // one parents
          1,
          // Parachain(0)
          [x1_parachain_asset_enum_selector + x1_parachain_id],
        ];

        const xcmMessage = {
          V2: [
            {
              ClearOrigin: null,
            },
          ],
        };

        const sentMessage: XcmVersionedXcm = context
          .polkadotJs()
          .createType("XcmVersionedXcm", xcmMessage) as any;

        // Try sending it with local view
        const localRawTxn = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmSend",
          args: [destHere, sentMessage.toHex()],
          rawTxOnly: true,
          gas: 1_000_000n,
        });

        const { result: localResult } = await context.createBlock(localRawTxn);
        expectEVMResult(localResult!.events, "Revert");
        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "XcmUtils",
              functionName: "xcmSend",
              args: [destHere, sentMessage.toHex()],
            })
        ).rejects.toThrowError(
          "Dispatched call failed with error: Module(ModuleError " +
            '{ index: 28, error: [0, 0, 0, 0], message: Some("Unreachable") })'
        );

        const paraRawTxn = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmSend",
          args: [destParaRelayView, sentMessage.toHex()],
          rawTxOnly: true,
          gas: 1_000_000n,
        });

        const { result: paraResult } = await context.createBlock(paraRawTxn);

        expectEVMResult(paraResult!.events, "Revert");
        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "XcmUtils",
              functionName: "xcmSend",
              args: [destParaRelayView, sentMessage.toHex()],
            })
        ).rejects.toThrowError(
          "Dispatched call failed with error: Module(ModuleError " +
            '{ index: 28, error: [0, 0, 0, 0], message: Some("Unreachable") })'
        );

        const paraRawTxn2 = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmSend",
          args: [destParaOtherParaView, sentMessage.toHex()],
          rawTxOnly: true,
          gas: 1_000_000n,
        });

        const { result: paraResult2 } = await context.createBlock(paraRawTxn2);

        expectEVMResult(paraResult2!.events, "Revert");
        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "XcmUtils",
              functionName: "xcmSend",
              args: [destParaOtherParaView, sentMessage.toHex()],
            })
        ).rejects.toThrowError(
          "Dispatched call failed with error: Module(ModuleError " +
            '{ index: 28, error: [1, 0, 0, 0], message: Some("SendFailure") })'
        );
      },
    });

    it({
      id: "T09",
      title: "allows to send a custom xcm message",
      test: async function () {
        // Sending it to the relay
        // { parents:1, Here}
        const dest = [
          // one parents
          1,
          // Here
          [],
        ];

        const xcmMessage = {
          V2: [
            {
              ClearOrigin: null,
            },
          ],
        };

        const sentMessage: XcmVersionedXcm = context
          .polkadotJs()
          .createType("XcmVersionedXcm", xcmMessage);

        const rawTxn = await context.writePrecompile!({
          precompileName: "XcmUtils",
          functionName: "xcmSend",
          args: [dest, sentMessage.toHex()],
          rawTxOnly: true,
          gas: 1_000_000n,
        });

        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");
      },
    });
  },
});
