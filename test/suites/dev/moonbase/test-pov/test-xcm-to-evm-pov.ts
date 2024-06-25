import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { Abi, encodeFunctionData } from "viem";
import { HeavyContract, deployHeavyContracts, expectOk } from "../../../../helpers";

import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress20,
  injectHrmpMessage,
} from "../../../../helpers/xcm.js";
import { GAS_LIMIT_POV_RATIO } from "../../../../helpers/constants";

describeSuite({
  id: "D012706",
  title: "XCM to EVM - PoV tests",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let transferredBalance;
    let sendingAddress: `0x${string}`;
    let proxyAbi: Abi;
    let proxyAddress: `0x${string}`;
    const MAX_CONTRACTS = 15;
    let contracts: HeavyContract[];
    const EXPECTED_POV_ROUGH = 350_000; // bytes
    let balancesPalletIndex: number;

    beforeAll(async function () {
      // Get Pallet balances index
      const metadata = await context.polkadotJs().rpc.state.getMetadata();
      const foundPallet = metadata.asLatest.pallets.find(
        (pallet) => pallet.name.toString() === "Balances"
      );

      if (!foundPallet || !foundPallet.index) {
        throw new Error("Balances pallet or its index not found");
      }

      balancesPalletIndex = foundPallet.index.toNumber();

      // Get derived account
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      transferredBalance = 10_000_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovreign account
      await expectOk(
        context.createBlock(
          context
            .polkadotJs()
            .tx.balances.transferAllowDeath(descendOriginAddress, transferredBalance)
        )
      );
      const balance = (
        (await context.polkadotJs().query.system.account(descendOriginAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);

      const { abi, contractAddress } = await deployCreateCompiledContract(context, "CallForwarder");
      proxyAbi = abi;
      proxyAddress = contractAddress;
      contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);
    });

    it({
      id: "T01",
      title: "should fail to execute evm tx with insufficient gas to cover PoV",
      test: async function () {
        const GAS_LIMIT = 2_000_000;
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
                Call: proxyAddress,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: proxyAbi,
                functionName: "callRange",
                args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
              }),
              access_list: null,
            },
          },
        ];

        const targetXcmWeight = BigInt(GAS_LIMIT) * 25000n + 25_000_000n + 800000000n;
        const targetXcmFee = targetXcmWeight * 50_000n;
        const transferCall = context
          .polkadotJs()
          .tx.ethereumXcm.transact(xcmTransactions[0] as any);
        const transferCallEncoded = transferCall?.method.toHex();

        // Build the XCM message
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: targetXcmFee,
            },
          ],
          weight_limit: {
            refTime: targetXcmWeight,
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
                refTime: 50_025_000_000,
                proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v3();

        // Send an XCM and create block to execute it
        await injectHrmpMessage(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);
        // This block is the one that processes the xcm messages
        const { result, block } = await context.createBlock();

        // With 500k gas we are allowed to use ~150k of POV, so verify the range.
        // The tx is still included in the block because it contains the failed tx,
        // so POV is included in the block as well.
        expect(block.proofSize).to.be.at.least(130_000);
        expect(block.proofSize).to.be.at.most(190_000);

        // Check the evm tx was not executed because of OutOfGas error
        const ethEvents = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.ethereum.Executed.is(event)
        );
        expect(ethEvents).to.have.lengthOf(1);
        expect((ethEvents[0].toHuman() as any).event["data"]["exitReason"]["Error"]).equals(
          "OutOfGas"
        );
      },
    });

    it({
      id: "T02",
      title: "should execute evm tx with enough gas to cover PoV",
      test: async function () {
        // Note: we can't use more than 6.4M gas through an XCM message, because it makes the entire
        // message weight to go over the allowed weight to execute an XCM message. This is called
        // "overweight".
        //
        // If we use more than 6.4M gas, we receive the "WeightLimitReached" error and
        // "OverweightEnqueued" event from the xcmpQueue pallet.
        const GAS_LIMIT = 6_400_000;
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
                Call: proxyAddress,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: proxyAbi,
                functionName: "callRange",
                args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
              }),
              access_list: null,
            },
          },
        ];

        const targetXcmWeight = BigInt(GAS_LIMIT) * 25000n + 25_000_000n + 800000000n;
        const targetXcmFee = targetXcmWeight * 50_000n;
        const transferCall = context
          .polkadotJs()
          .tx.ethereumXcm.transact(xcmTransactions[0] as any);
        const transferCallEncoded = transferCall?.method.toHex();
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: targetXcmFee,
            },
          ],
          weight_limit: {
            refTime: targetXcmWeight,
            proofSize: (GAS_LIMIT / GAS_LIMIT_POV_RATIO) * 2,
          },
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: 160_025_000_000,
                proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
              },
              call: {
                encoded: transferCallEncoded,
              },
            },
          })
          .as_v3();

        // Send an XCM and create block to execute it
        await injectHrmpMessage(context, 1, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        } as RawXcmMessage);

        // This block is the one that processes the xcm messages
        const { result, block } = await context.createBlock();

        expect(block.proofSize).to.be.at.least(EXPECTED_POV_ROUGH / 1.1);
        expect(block.proofSize).to.be.at.most(EXPECTED_POV_ROUGH * 1.1);

        // Check the evm tx was executed successfully
        const ethEvents = (await context.polkadotJs().query.system.events()).filter(({ event }) =>
          context.polkadotJs().events.ethereum.Executed.is(event)
        );
        expect(ethEvents).to.have.lengthOf(1);
        expect((ethEvents[0].toHuman() as any).event["data"]["exitReason"]["Succeed"]).equals(
          "Stopped"
        );
      },
    });
  },
});
