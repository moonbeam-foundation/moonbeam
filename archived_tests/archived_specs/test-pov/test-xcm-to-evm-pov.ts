import "@moonbeam-network/api-augment";

import { expect } from "chai";
import {
  descendOriginFromAddress20,
  RawXcmMessage,
  XcmFragment,
  injectHrmpMessage,
} from "../../util/xcm";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { expectOk } from "../../util/expect";
import { Contract } from "web3-eth-contract";
import { GAS_LIMIT_POV_RATIO } from "../../util/constants";
import { deployHeavyContracts, HeavyContract } from "./test-evm-over-pov";

describeDevMoonbeam("XCM to EVM - PoV tests", (context) => {
  let transferredBalance;
  let sendingAddress;
  let contractProxy: Contract;
  const MAX_CONTRACTS = 15;
  let contracts: HeavyContract[];
  const EXPECTED_POV_ROUGH = 350_000; // bytes
  let balancesPalletIndex;
  let metadata;

  before("Deploy the contracts from range 6000-6015", async function () {
    // Get Pallet balances index
    metadata = await context.polkadotApi.rpc.state.getMetadata();
    balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find((pallet) => {
      return pallet.name === "Balances";
    }).index;

    // Get derived account
    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    sendingAddress = originAddress;
    transferredBalance = 10_000_000_000_000_000_000_000n;

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

    // Deploy the CallForwarder contract
    const creation = await createContract(context, "CallForwarder");
    contractProxy = creation.contract;
    await context.createBlock(creation.rawTx);

    // Deploy heavy contracts
    contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);
  });

  it("should fail to execute evm tx with insufficient gas to cover PoV", async function () {
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
            Call: contractProxy.options.address,
          },
          value: 0n,
          input: contractProxy.methods
            .callRange(contracts[0].account, contracts[MAX_CONTRACTS].account)
            .encodeABI()
            .toString(),
          access_list: null,
        },
      },
    ];

    const targetXcmWeight = 500_000n * 25000n + 25_000_000n + 800000000n;
    const targetXcmFee = targetXcmWeight * 50_000n;
    const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransactions[0] as any);
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
    expect(block.proof_size).to.be.at.least(130_000);
    expect(block.proof_size).to.be.at.most(190_000);

    // Check the evm tx was not executed because of OutOfGas error
    const ethEvents = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
      context.polkadotApi.events.ethereum.Executed.is(event)
    );
    expect(ethEvents).to.have.lengthOf(1);
    expect(ethEvents[0].toHuman().event["data"]["exitReason"]["Error"]).equals("OutOfGas");
  });

  it("should execute evm tx with enough gas to cover PoV", async function () {
    // Note: we can't use more than 1.6M gas through an XCM message, because it makes the entire
    // message weight to go over the allowed weight to execute an XCM message. This is called
    // "overweight".
    //
    // If we use more than 1.6M gas, we receive the "WeightLimitReached" error and
    // "OverweightEnqueued" event from the xcmpQueue pallet.
    const GAS_LIMIT = 1_600_000;
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
            Call: contractProxy.options.address,
          },
          value: 0n,
          input: contractProxy.methods
            .callRange(contracts[0].account, contracts[MAX_CONTRACTS].account)
            .encodeABI()
            .toString(),
          access_list: null,
        },
      },
    ];

    const targetXcmWeight = 1_600_000n * 25000n + 25_000_000n + 800000000n;
    const targetXcmFee = targetXcmWeight * 50_000n;
    const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransactions[0] as any);
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
            refTime: 40_025_000_000,
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

    expect(block.proof_size).to.be.at.least(EXPECTED_POV_ROUGH / 1.1);
    expect(block.proof_size).to.be.at.most(EXPECTED_POV_ROUGH * 1.1);

    // Check the evm tx was executed successfully
    const ethEvents = (await context.polkadotApi.query.system.events()).filter(({ event }) =>
      context.polkadotApi.events.ethereum.Executed.is(event)
    );
    expect(ethEvents).to.have.lengthOf(1);
    expect(ethEvents[0].toHuman().event["data"]["exitReason"]["Succeed"]).equals("Stopped");
  });
});
