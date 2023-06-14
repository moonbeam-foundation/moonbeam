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
} from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { GAS_LIMIT_POV_RATIO } from "../../util/constants";

import { createContract } from "../../util/transactions";

import { expectOk } from "../../util/expect";

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (transfer)", (context) => {
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
      await injectHrmpMessageAndSeal(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

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

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (call)", (context) => {
  let transferredBalance;
  let sendingAddress;
  let random: KeyringPair;
  let contractDeployed;

  before("should receive transact action with DescendOrigin and deploy", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await expectOk(context.createBlock(rawTx));

    contractDeployed = contract;

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    sendingAddress = originAddress;
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
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: contractDeployed.methods.incr().encodeABI().toString(),
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: 100000,
          action: {
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: contractDeployed.methods.incr().encodeABI().toString(),
          access_list: null,
        },
      },
    ];

    let expectedCalls = 0n;

    for (const xcmTransaction of xcmTransactions) {
      expectedCalls++;

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

      expect(await contractDeployed.methods.count().call()).to.eq(expectedCalls.toString());
    }
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (asset fee)", (context) => {
  const assetMetadata = {
    name: "FOREIGN",
    symbol: "FOREIGN",
    decimals: new BN(12),
    isFrozen: false,
  };
  const statemint_para_id = 1001;
  const statemint_assets_pallet_instance = 50;
  const palletId = "0x6D6f646c617373746d6E67720000000000000000";

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

  let assetId: string;
  let sendingAddress;
  let descendedAddress: string;
  let random: KeyringPair;
  let contractDeployed;

  // Gas limit + one db read
  const assetsToTransfer = (3_300_000_000n + 25_000_000n) * 2n;

  before("should Register an asset and set unit per sec", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await expectOk(context.createBlock(rawTx));

    contractDeployed = contract;

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    sendingAddress = originAddress;
    descendedAddress = descendOriginAddress;
    random = generateKeyringPair();

    // registerForeignAsset
    const { registeredAssetId, events, registeredAsset } = await registerForeignAsset(
      context,
      STATEMINT_LOCATION,
      assetMetadata,
      1_000_000_000_000
    );
    assetId = registeredAssetId;
    expect(events[1].event.method.toString()).to.eq("UnitsPerSecondChanged");
    expect(events[5].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

    let config = {
      assets: [
        {
          multilocation: ASSET_MULTILOCATION,
          fungible: 0n,
        },
      ],
      beneficiary: descendOriginAddress,
    };

    // How much will the message weight?
    const chargedWeight = await weightMessage(
      context,
      context.polkadotApi.createType(
        "XcmVersionedXcm",
        new XcmFragment(config)
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution()
          .deposit_asset()
          .as_v2()
      ) as any
    );

    // we modify the config now:
    // we send assetsToTransfer plus whatever we will be charged in weight
    config.assets[0].fungible = assetsToTransfer + chargedWeight;

    // Construct the real message
    const xcmMessage = new XcmFragment(config)
      .reserve_asset_deposited()
      .clear_origin()
      .buy_execution()
      .deposit_asset()
      .as_v2();

    // Send an XCM and create block to execute it
    await injectHrmpMessageAndSeal(context, statemint_para_id, {
      type: "XcmVersionedXcm",
      payload: xcmMessage,
    } as RawXcmMessage);

    // Make sure descended address has the transferred foreign assets (minus the xcm fees).
    expect(
      (await context.polkadotApi.query.assets.account(assetId, descendedAddress))
        .unwrap()
        .balance.toBigInt()
    ).to.eq(assetsToTransfer);
  });

  it("should receive transact and should be able to execute", async function () {
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
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: contractDeployed.methods.incr().encodeABI().toString(),
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: 100000,
          action: {
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: contractDeployed.methods.incr().encodeABI().toString(),
          access_list: null,
        },
      },
    ];

    let expectedCalls = 0n;

    for (const xcmTransaction of xcmTransactions) {
      expectedCalls++;

      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction as any);
      const transferCallEncoded = transferCall?.method.toHex();

      // We are going to test that we can receive a transact operation from parachain 1
      // using descendOrigin first
      const xcmMessage = new XcmFragment({
        assets: [
          {
            multilocation: ASSET_MULTILOCATION,
            fungible: assetsToTransfer / 2n,
          },
        ],
        weight_limit: new BN((assetsToTransfer / 2n).toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            // 100_000 gas + 1 db read
            requireWeightAtMost: new BN(2_500_000_000).add(new BN(25_000_000)),
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

      expect(await contractDeployed.methods.count().call()).to.eq(expectedCalls.toString());
    }
    // Make sure descended address went below existential deposit and was killed
    expect((await context.polkadotApi.query.assets.account(assetId, descendedAddress)).isNone).to.be
      .true;
    // Even if the account does not exist in assets aymore, we still have a nonce 1. Reason is:
    //  - First transact withdrew 1/2 of assets, nonce was increased to 1.
    //  - Second transact withdrew the last 1/2 of assets, account was reaped and zeroed.
    //  - The subsequent evm execution increased the nonce to 1, even without sufficient references.
    // We can expect this to be the behaviour on any xcm fragment that completely drains an account
    // to transact ethereum-xcm after.
    let nonce = await context.web3.eth.getTransactionCount(descendedAddress);
    expect(nonce).to.be.eq(1);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (proxy)", (context) => {
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

    // We first fund the descend origin derivated address
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

  it("should fail to transact_through_proxy without proxy", async function () {
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

    let feeAmount = 0n;

    // Gas limit + 2 db reads
    const targetXcmWeight = 1_325_000_000n + 100_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      feeAmount += targetXcmFee;
      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transactThroughProxy(
        sendingAddress,
        xcmTransaction
      );
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
            // 100_000 gas + 2 db read
            requireWeightAtMost: new BN(525_000_000).add(new BN(50_000_000)),
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

      // Make sure the state for the transfer recipient didn't change
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(0n);

      // Make sure the descended address has been deducted fees once (in xcm-executor) but
      // transfered nothing.
      const descendOriginBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(descendOriginBalance)).to.eq(transferredBalance - feeAmount);
    }
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (proxy)", (context) => {
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("should receive transact action with DescendOrigin", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
      context,
      charleth.address
    );
    sendingAddress = originAddress;
    descendAddress = descendOriginAddress;
    random = generateKeyringPair();
    transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund the descend origin derivated address
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendOriginAddress, transferredBalance)
      )
    );
    const balance = (
      (await context.polkadotApi.query.system.account(descendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);

    // Add proxy with delay 1
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(descendAddress, "Any" as any, 1).signAsync(charleth)
    );
  });

  it("should fail to transact_through_proxy with non-zero delay proxy", async function () {
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

    let feeAmount = 0n;

    const targetXcmWeight = 1_325_000_000n + 100_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      feeAmount += targetXcmFee;
      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transactThroughProxy(
        sendingAddress,
        xcmTransaction
      );
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
            // 100_000 gas + 2 reads
            requireWeightAtMost: new BN(525_000_000).add(new BN(50_000_000)),
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

      // Make sure the state for the transfer recipient didn't change
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(0n);

      // Make sure the descended address has been deducted fees once (in xcm-executor) but
      // transfered nothing.
      const descendOriginBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(descendOriginBalance)).to.eq(transferredBalance - feeAmount);
    }
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (proxy)", (context) => {
  let charlethBalance;
  let charlethNonce;
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
      context,
      charleth.address
    );
    sendingAddress = originAddress;
    descendAddress = descendOriginAddress;
    random = generateKeyringPair();
    transferredBalance = 10_000_000_000_000_000_000n;

    // We fund the Delegatee, which will send the xcm and pay fees
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendAddress, transferredBalance)
      )
    );

    // Ensure funded
    const balance_delegatee = (
      (await context.polkadotApi.query.system.account(descendAddress)) as any
    ).data.free.toBigInt();
    expect(balance_delegatee).to.eq(transferredBalance);

    // Add proxy
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(descendAddress, "Any" as any, 0).signAsync(charleth)
    );

    // Charleth balance after creating the proxy
    charlethBalance = (
      (await context.polkadotApi.query.system.account(sendingAddress)) as any
    ).data.free.toBigInt();

    // Charleth nonce
    charlethNonce = parseInt(
      ((await context.polkadotApi.query.system.account(sendingAddress)) as any).nonce
    );
  });

  it("should succeed to transact_through_proxy with proxy", async function () {
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

    const targetXcmWeight = 1_325_000_000n + 100_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      expectedTransferredAmount += amountToTransfer;
      expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transactThroughProxy(
        sendingAddress,
        xcmTransaction
      );
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
            // 100_000 gas + 2db reads
            requireWeightAtMost: new BN(525_000_000).add(new BN(50_000_000)),
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

      // The transfer destination
      // Make sure the destination address received the funds
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(expectedTransferredAmount);

      // The EVM caller (proxy delegator)
      // Make sure CHARLETH called the evm on behalf DESCENDED, and CHARLETH balance was deducted.
      const charlethAccountBalance = await context.web3.eth.getBalance(sendingAddress);
      expect(BigInt(charlethAccountBalance)).to.eq(charlethBalance - expectedTransferredAmount);
      // Make sure CHARLETH nonce was increased, as EVM caller.
      const charlethAccountNonce = await context.web3.eth.getTransactionCount(sendingAddress);
      expect(charlethAccountNonce).to.eq(charlethNonce + 1);
      charlethNonce++;

      // The XCM sender (proxy delegatee)
      // Make sure derived / descended account paid the xcm fees only.
      const derivedAccountBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(derivedAccountBalance)).to.eq(
        transferredBalance - (expectedTransferredAmountPlusFees - expectedTransferredAmount)
      );
      // Make sure derived / descended account nonce still zero.
      const derivedAccountNonce = await context.web3.eth.getTransactionCount(descendAddress);
      expect(derivedAccountNonce).to.eq(0);
    }
  });
});

describeDevMoonbeam("Mock XCM - transact ETHEREUM (proxy) disabled switch", (context) => {
  let charlethBalance;
  let charlethNonce;
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
      context,
      charleth.address
    );
    sendingAddress = originAddress;
    descendAddress = descendOriginAddress;
    random = generateKeyringPair();
    transferredBalance = 10_000_000_000_000_000_000n;

    // We fund the Delegatee, which will send the xcm and pay fees
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(descendAddress, transferredBalance)
      )
    );

    // Ensure funded
    const balance_delegatee = (
      (await context.polkadotApi.query.system.account(descendAddress)) as any
    ).data.free.toBigInt();
    expect(balance_delegatee).to.eq(transferredBalance);

    // Add proxy
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(descendAddress, "Any" as any, 0).signAsync(charleth)
    );

    // Charleth balance after creating the proxy
    charlethBalance = (
      (await context.polkadotApi.query.system.account(sendingAddress)) as any
    ).data.free.toBigInt();

    // Charleth nonce
    charlethNonce = parseInt(
      ((await context.polkadotApi.query.system.account(sendingAddress)) as any).nonce
    );

    // We activate the suspension switch
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.ethereumXcm.suspendEthereumXcmExecution())
        .signAsync(alith)
    );
  });

  it("should fail to transact_through_proxy with proxy when disabled", async function () {
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

    const targetXcmWeight = 1_325_000_000n + 100_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      expectedTransferredAmount += amountToTransfer;
      expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transactThroughProxy(
        sendingAddress,
        xcmTransaction
      );
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
            // 100_000 gas + 2db reads
            requireWeightAtMost: new BN(525_000_000).add(new BN(50_000_000)),
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

      // The transfer destination
      // Make sure the destination address did not receive the funds
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(0n);

      // The EVM caller (proxy delegator)
      // Make sure CHARLETH balance was not deducted.
      const charlethAccountBalance = await context.web3.eth.getBalance(sendingAddress);
      expect(BigInt(charlethAccountBalance)).to.eq(charlethBalance);
      // Make sure CHARLETH nonce did not increase.
      const charlethAccountNonce = await context.web3.eth.getTransactionCount(sendingAddress);
      expect(charlethAccountNonce).to.eq(charlethNonce);

      // The XCM sender (proxy delegatee)
      // Make sure derived / descended account paid the xcm fees only.
      const derivedAccountBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(derivedAccountBalance)).to.eq(
        transferredBalance - (expectedTransferredAmountPlusFees - expectedTransferredAmount)
      );
      // Make sure derived / descended account nonce still zero.
      const derivedAccountNonce = await context.web3.eth.getTransactionCount(descendAddress);
      expect(derivedAccountNonce).to.eq(0);
    }
  });
});

describeDevMoonbeam("Mock XCM - transact ETHEREUM (non-proxy) disabled switch", (context) => {
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

    // We activate the suspension switch
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.ethereumXcm.suspendEthereumXcmExecution())
        .signAsync(alith)
    );
  });

  it("should receive transact and should not be able to execute", async function () {
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

    let expectedTransferredAmountPlusFees = 0n;

    const targetXcmWeight = 1_325_000_000n + 25_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      expectedTransferredAmountPlusFees += targetXcmFee;
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
      await injectHrmpMessageAndSeal(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      // Make sure tokens have not bein transferred
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(0n);

      // Make sure descend address has been deducted fees once (in xcm-executor)
      const descendAddressBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(descendAddressBalance)).to.eq(
        transferredBalance - expectedTransferredAmountPlusFees
      );
    }
  });
});

describeDevMoonbeam("Mock XCM - EthereumXcm only disable by root", (context) => {
  it("should check suspend ethereum xcm only callable by root", async function () {
    let suspended = await context.polkadotApi.query.ethereumXcm.ethereumXcmSuspended();
    // should be not suspended by default
    expect(suspended.toHuman()).to.be.false;

    // We try to activate without sudo
    await context.createBlock(
      context.polkadotApi.tx.ethereumXcm.suspendEthereumXcmExecution().signAsync(alith)
    );
    suspended = await context.polkadotApi.query.ethereumXcm.ethereumXcmSuspended();
    // should not have worked, and should still not be suspended
    expect(suspended.toHuman()).to.be.false;

    // Now with sudo
    await context.createBlock(
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.ethereumXcm.suspendEthereumXcmExecution())
        .signAsync(alith)
    );

    suspended = await context.polkadotApi.query.ethereumXcm.ethereumXcmSuspended();
    // should have worked, and should now be suspended
    expect(suspended.toHuman()).to.be.true;
  });
});

describeDevMoonbeam("Mock XCM - transact ETHEREUM input size check succeeds", (context) => {
  let transferredBalance;
  let sendingAddress;
  let contractDeployed;

  before("should deploy CallForwarder contract and fund", async function () {
    const { contract, rawTx } = await createContract(context, "CallForwarder");
    await expectOk(context.createBlock(rawTx));

    contractDeployed = contract;

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    sendingAddress = originAddress;
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

  it("should succeed to call the contract", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
    // Matches the BoundedVec limit in the runtime.
    const CALL_INPUT_SIZE_LIMIT = Math.pow(2, 16);

    const GAS_LIMIT = 1000000;

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
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: proxyInterface.encodeFunctionData("call", [
            "0x0000000000000000000000000000000000000001",
            context.web3.utils.bytesToHex(new Array(CALL_INPUT_SIZE_LIMIT - 128).fill(0)),
          ]),
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: GAS_LIMIT,
          action: {
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: proxyInterface.encodeFunctionData("call", [
            "0x0000000000000000000000000000000000000001",
            context.web3.utils.bytesToHex(new Array(CALL_INPUT_SIZE_LIMIT - 128).fill(0)),
          ]),
          access_list: null,
        },
      },
    ];

    for (const xcmTransaction of xcmTransactions) {
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
            fungible: transferredBalance / 2n,
          },
        ],
        weight_limit: {
          refTime: 40000000000,
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
              refTime: 30000000000,
              proofSize: GAS_LIMIT / GAS_LIMIT_POV_RATIO,
            },
            call: {
              encoded: transferCallEncoded,
            },
          },
        })
        .as_v3();

      // Send an XCM and create block to execute it
      await injectHrmpMessageAndSeal(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      const block = await context.web3.eth.getBlock("latest");
      // Input size is valid - on the limit -, expect block to include a transaction.
      // That means the pallet-ethereum-xcm decoded the provided input to a BoundedVec.
      expect(block.transactions.length).to.be.eq(1);
    }
  });
});

describeDevMoonbeam("Mock XCM - transact ETHEREUM input size check fails", (context) => {
  let transferredBalance;
  let sendingAddress;
  let contractDeployed;

  before("should deploy CallForwarder contract and fund", async function () {
    const { contract, rawTx } = await createContract(context, "CallForwarder");
    await expectOk(context.createBlock(rawTx));

    contractDeployed = contract;

    const { originAddress, descendOriginAddress } = descendOriginFromAddress20(context);
    sendingAddress = originAddress;
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

  it("should fail to call the contract due to BoundedVec restriction", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);
    // Matches the BoundedVec limit in the runtime.
    const CALL_INPUT_SIZE_LIMIT = Math.pow(2, 16);

    const xcmTransactions = [
      {
        V1: {
          gas_limit: 1000000,
          fee_payment: {
            Auto: {
              Low: null,
            },
          },
          action: {
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: proxyInterface.encodeFunctionData("call", [
            "0x0000000000000000000000000000000000000001",
            context.web3.utils.bytesToHex(new Array(CALL_INPUT_SIZE_LIMIT - 127).fill(0)),
          ]),
          access_list: null,
        },
      },
      {
        V2: {
          gas_limit: 1000000,
          action: {
            Call: contractDeployed.options.address,
          },
          value: 0n,
          input: proxyInterface.encodeFunctionData("call", [
            "0x0000000000000000000000000000000000000001",
            context.web3.utils.bytesToHex(new Array(CALL_INPUT_SIZE_LIMIT - 127).fill(0)),
          ]),
          access_list: null,
        },
      },
    ];

    for (const xcmTransaction of xcmTransactions) {
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
            fungible: transferredBalance / 2n,
          },
        ],
        weight_limit: new BN(40000000000),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(30000000000),
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

      const block = await context.web3.eth.getBlock("latest");
      // Input size is invalid by 1 byte, expect block to not include a transaction.
      // That means the pallet-ethereum-xcm couldn't decode the provided input to a BoundedVec.
      expect(block.transactions.length).to.be.eq(0);
    }
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (transfer)", (context) => {
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("should receive ethereum transact and account weight used", async function () {
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

  it("should receive transact and should use less weight than gas limit", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const amountToTransfer = transferredBalance / 10n;

    const GAS_LIMIT = 500_000;

    // We will put a very high gas limit. However, the weight accounted
    // for the block should only
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

    const targetXcmWeight = 500_000n * 25000n + 25_000_000n + 800000000n;
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
            // 500_000 gas limit + db read
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
      await injectHrmpMessageAndSeal(context, 1, {
        type: "XcmVersionedXcm",
        payload: xcmMessage,
      } as RawXcmMessage);

      // Make sure the state has ALITH's foreign parachain tokens
      const testAccountBalance = (
        await context.polkadotApi.query.system.account(random.address)
      ).data.free.toBigInt();
      expect(testAccountBalance).to.eq(expectedTransferredAmount);

      // Make sure ALITH has been deducted fees once (in xcm-executor) and balance has been
      // transfered through evm.
      const alithAccountBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(alithAccountBalance)).to.eq(
        transferredBalance - expectedTransferredAmountPlusFees
      );

      const weightBlock = await context.polkadotApi.query.system.blockWeight();
      // Make sure the system block weight corresponds to gas used and not gas limit
      // It should be sufficient to verify that we used less than what was marked
      expect(12_500_000_000n + 25_000_000n - weightBlock.mandatory.refTime.toBigInt() > 0n).to.be
        .true;
    }
  });
});
