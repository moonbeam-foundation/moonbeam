import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction, GAS_LIMIT_POV_RATIO } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../helpers/eth-transactions.ts";
import { expectOk } from "../../../helpers/expect.ts";
import {
  descendOriginFromAddress20,
  injectHrmpMessage,
  RawXcmMessage,
  XcmFragment,
} from "../../../helpers/xcm.ts";
import * as console from "console";

describeSuite({
  id: "D2406",
  title: "PoV controlled by gasLimit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let transferredBalance;
    let storageLoopAddress: `0x${string}`;
    let storageLoopAbi: Abi;
    let balancesPalletIndex: number;
    let sendingAddress: `0x${string}`;

    beforeAll(async () => {
      const metadata = await context.polkadotJs().rpc.state.getMetadata();

      balancesPalletIndex = metadata.asLatest.pallets
        .find((pallet) => {
          return pallet.name.toString() === "Balances";
        })
        ?.index.toNumber()!;

      // Get derived account
      const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
      sendingAddress = originAddress;
      transferredBalance = 10_000_000_000_000_000_000_000n;

      // We first fund parachain 2000 sovreign account
      await expectOk(
        context.createBlock(
          context.polkadotJs().tx.balances.transfer(descendOriginAddress, transferredBalance)
        )
      );
      const balance = (
        (await context.polkadotJs().query.system.account(descendOriginAddress)) as any
      ).data.free.toBigInt();
      expect(balance).to.eq(transferredBalance);

      const { contractAddress, abi } = await deployCreateCompiledContract(context, "Storage");
      storageLoopAddress = contractAddress;
      storageLoopAbi = abi;

      await context.createBlock();
      let nonce = (
        await context.polkadotJs().rpc.system.accountNextIndex(ALITH_ADDRESS)
      ).toNumber();

      for (let i = 0; i < 2000; i++) {
        const rawSigned = await createEthersTransaction(context, {
          to: storageLoopAddress,
          data: encodeFunctionData({
            abi: storageLoopAbi,
            functionName: "store",
            // Add 200 new storage entries
            args: [i * 50, (i + 1) * 50],
          }),
          gasLimit: 10_000_000,
          nonce: nonce++,
        });
        await context.createBlock(rawSigned, { allowFailures: false });
      }
      let block_number = (await context.polkadotJs().rpc.chain.getHeader()).number.toNumber();
    });

    // it({
    //   id: `T01`,
    //   title: "PoV exceeds Limit",
    //   test: async function () {
    //     const rawTx = await createEthersTransaction(context, {
    //       to: storageLoopAddress,
    //       data: encodeFunctionData({
    //         abi: storageLoopAbi,
    //         functionName: "destroy",
    //       }),
    //       gasLimit: 200_000,
    //     });
    //     const { result, block } = await context.createBlock(rawTx);
    //     log(`block.proof_size: ${block.proofSize}`);
    //     // Get gas used
    //     expectEVMResult(result!.events, "Succeed");
    //   },
    // });

    it({
      id: "T01",
      title: "should not exceed PoV Limit",
      test: async function () {
        const GAS_LIMIT = 500_000;
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
                Call: storageLoopAddress,
              },
              value: 0n,
              input: encodeFunctionData({
                abi: storageLoopAbi,
                functionName: "destroy",
              }),
              access_list: null,
            },
          },
        ];

        const targetXcmWeight = 500_000n * 25000n + 25_000_000n + 800000000n;
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
                refTime: 12_525_000_000,
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
        const { result, block } = await context.createBlock();

        // With 500k gas we are allowed to use ~150k of POV, so verify the range.
        // The tx is still included in the block because it contains the failed tx,
        // so POV is included in the block as well.
        console.log("block.proofSize", block.proofSize);
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
  },
});
