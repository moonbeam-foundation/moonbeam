import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect, deployCreateCompiledContract } from "@moonwall/cli";
import { Abi, encodeFunctionData } from "viem";
import { DummyContract, deployAccountCodesMetadata, expectOk } from "../../../../helpers";

import {
  RawXcmMessage,
  XcmFragment,
  descendOriginFromAddress20,
  injectHrmpMessage,
} from "../../../../helpers/xcm.js";
import { GAS_LIMIT_POV_RATIO } from "@moonwall/util";

describeSuite({
  id: "D012607",
  title: "XCM to EVM - PoV tests 2",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let transferredBalance;
    let sendingAddress: `0x${string}`;
    let proxyAbi: Abi;
    let proxyAddress: `0x${string}`;
    const MAX_CONTRACTS = 1_000;
    let contracts: DummyContract[];
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
      transferredBalance = 1_000_000_000_000_000_000_000_000n;

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

      const { abi, contractAddress } = await deployCreateCompiledContract(context, "ExtCodeSizeRange");
      proxyAbi = abi;
      proxyAddress = contractAddress;
      contracts = await deployAccountCodesMetadata(context, 6000, 6000 + MAX_CONTRACTS);
    });

    it({
      id: "T01",
      title: "should fail to execute evm tx with insufficient gas to cover PoV",
      test: async function () {
        const GAS_LIMIT = 3_750_000;
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
                functionName: "range",
                args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
              }),
              access_list: null,
            },
          },
        ];

        const targetXcmWeight = BigInt(GAS_LIMIT) * 25000n + 1_000_000_000n;
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
            refTime: 110_000_000_000n,
            proofSize: 1_000_000,
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
                refTime: 100_000_000_000n,
                proofSize: 965_325,
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
        const { result, block } = await context.createBlock();

        // Ensure that the PoV still reasonable (under 1Mb)
        expect(block.proofSize).to.be.at.most(1 * 1_024 * 1_024);
      },
    });
  },
});
