import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { BN } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";

import { alith, charleth, generateKeyringPair } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import {
  descendOriginFromAddress20,
  registerForeignAsset,
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  XcmFragment,
  weightMessage,
  MultiLocation,
  injectHrmpMessage
} from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { createContract } from "../../util/transactions";

import { expectOk } from "../../util/expect";

describeDevMoonbeam("XCM to EVM - PoV tests", (context) => {
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("should receive transact action with DescendOrigin", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    sendingAddress = originAddress;
    descendAddress = descendOriginAddress;
    random = generateKeyringPair();
    transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendOriginAddress, transferredBalance)
      )
    );
    const balance = (
      (await context.polkadotApi.query.system.account(descendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("should receive transact and should be able to execute", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const amountToTransfer = transferredBalance / 10n;

    const xcmTransactions = [
      {
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
          value: amountToTransfer,
          input: [],
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: 21000,
          action: {
            Call: random.address,
          },
          value: amountToTransfer,
          input: [],
          access_list: null,
        },
      },
    ];

    let expectedTransferredAmount = 0n;
    let expectedTransferredAmountPlusFees = 0n;

    const targetXcmWeight = 1_325_000_000n + 25_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      expectedTransferredAmount += amountToTransfer;
      expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction as any);
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
            fungible: targetXcmFee,
          },
        ],
        weight_limit: new BN(targetXcmWeight.toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            // 21_000 gas limit + db read
            requireWeightAtMost: new BN(525_000_000).add(new BN(25_000_000)),
            call: {
              encoded: transferCallEncoded,
            },
          },
        })
        .as_v2();

      // Send an XCM and create block to execute it
      await injectHrmpMessage(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);
      
      const {result, block} = await context.createBlock();

      console.log("Proof size", block.proof_size);

      // Make sure the state has ALITH's foreign parachain tokens
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(expectedTransferredAmount);

      // Make sure descend address has been deducted fees once (in xcm-executor) and balance
      // has been transfered through evm.
      const descendAccountBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(descendAccountBalance)).to.eq(
        transferredBalance - expectedTransferredAmountPlusFees
      );
    }
  });
});
