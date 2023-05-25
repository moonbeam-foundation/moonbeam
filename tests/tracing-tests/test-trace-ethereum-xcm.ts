import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { BN } from "@polkadot/util";
import { expect } from "chai";

import { generateKeyringPair, alith } from "../util/accounts";
import {
  descendOriginFromAddress20,
  injectHrmpMessage,
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  XcmFragment,
} from "../util/xcm";

import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { createContract } from "../util/transactions";
import { customWeb3Request } from "../util/providers";

import { expectOk } from "../util/expect";

describeDevMoonbeam("Trace ethereum xcm #1", (context) => {
  let transactionHashes = [];

  before("should receive transact action with DescendOrigin", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await expectOk(context.createBlock(rawTx));

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    const sendingAddress = originAddress;
    const random = generateKeyringPair();
    const transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendOriginAddress, transferredBalance)
      )
    );
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

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
            Call: contract.options.address,
          },
          value: 0n,
          input: contract.methods.incr().encodeABI().toString(),
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: 100000,
          action: {
            Call: contract.options.address,
          },
          value: 0n,
          input: contract.methods.incr().encodeABI().toString(),
          access_list: null,
        },
      },
    ];

    for (const xcmTransaction of xcmTransactions) {
      const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction as any);
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
            requireWeightAtMost: new BN(3000000000),
            call: {
              encoded: transferCallEncoded,
            },
          },
        })
        .as_v2();

      // Send an XCM and create block to execute it
      await injectHrmpMessageAndSeal(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      // Retrieve the stored ethereum transaction hash
      transactionHashes.push((await context.web3.eth.getBlock("latest")).transactions[0]);
    }
  });

  it("should trace ethereum xcm transactions with debug_traceTransaction", async function () {
    for (const hash of transactionHashes) {
      const receipt = await context.web3.eth.getTransactionReceipt(hash);
      const trace = await customWeb3Request(context.web3, "debug_traceTransaction", [
        hash,
        { tracer: "callTracer" },
      ]);
      // We traced the transaction, and the traced gas used matches the one recorded
      // in the ethereum transaction receipt.
      expect(receipt.gasUsed).to.eq(context.web3.utils.hexToNumber(trace.result.gasUsed));
    }
  });
});

describeDevMoonbeam("Trace ethereum xcm #2", (context) => {
  let ethereumXcmDescendedOrigin;
  let xcmContractAddress;
  let ethContractAddress;

  before("should receive transact action with DescendOrigin", async function () {
    const { contract: xcm_contract, rawTx: xcm_rawTx } = await createContract(
      context,
      "Incrementor"
    );
    await expectOk(context.createBlock(xcm_rawTx));

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    ethereumXcmDescendedOrigin = descendOriginAddress;
    xcmContractAddress = xcm_contract.options.address;
    const sendingAddress = originAddress;
    const random = generateKeyringPair();
    const transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(ethereumXcmDescendedOrigin, transferredBalance)
      )
    );
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const xcmTransaction = {
      V2: {
        gas_limit: 100000,
        action: {
          Call: xcmContractAddress,
        },
        value: 0n,
        input: xcm_contract.methods.incr().encodeABI().toString(),
        access_list: null,
      },
    };

    const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction as any);
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
          requireWeightAtMost: new BN(3000000000),
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

    // Include a regular ethereum transaction
    const { contract, rawTx: eth_rawTx } = await createContract(context, "EventEmitter", {
      from: alith.address,
    });
    ethContractAddress = contract.options.address;
    // Create a block, it will include the ethereum xcm call + regular ethereum transaction
    await context.createBlock(eth_rawTx);
  });

  it("should trace ethereum xcm transactions with debug_traceBlockByNumber", async function () {
    const number = await context.web3.eth.getBlockNumber();
    const trace = (
      await customWeb3Request(context.web3, "debug_traceBlockByNumber", [
        number.toString(),
        { tracer: "callTracer" },
      ])
    ).result;
    // 2 ethereum transactions: ethereum xcm call + regular ethereum transaction
    expect(trace.length).to.eq(2);
    // 1st transaction is xcm.
    // - `From` is the descended origin.
    // - `To` is the xcm contract address.
    expect(trace[0].from).to.eq(ethereumXcmDescendedOrigin.toLowerCase());
    expect(trace[0].to).to.eq(xcmContractAddress.toLowerCase());
    expect(trace[0].type).to.eq("CALL");
    // 2nd transaction is regular ethereum transaction.
    // - `From` is Alith's adddress.
    // - `To` is the ethereum contract address.
    expect(trace[1].from).to.eq(alith.address.toLowerCase());
    expect(trace[1].to).to.eq(ethContractAddress.toLowerCase());
    expect(trace[1].type).be.eq("CREATE");
  });
});
