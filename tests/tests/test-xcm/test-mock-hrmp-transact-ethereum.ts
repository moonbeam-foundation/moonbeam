import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { BN } from "@polkadot/util";
import { expect } from "chai";

import { alith, charleth, generateKeyringPair } from "../../util/accounts";
import {
  descendOriginFromAddress,
  registerForeignAsset,
  injectHrmpMessageAndSeal,
  RawXcmMessage,
  XcmFragment,
} from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { createContract } from "../../util/transactions";

import { expectOk } from "../../util/expect";

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (transfer)", (context) => {
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("should receive transact action with DescendOrigin", async function () {
    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
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

    const targetXcmWeight = 1_325_000_000n;
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
        fees: {
          multilocation: [
            {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
          ],
          fungible: targetXcmFee,
        },
        weight_limit: new BN(targetXcmWeight.toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(525_000_000), // 21_000 gas limit
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

      // Make sure ALITH has been deducted fees once (in xcm-executor) and balance has been
      // transfered through evm.
      const alithAccountBalance = await context.web3.eth.getBalance(descendAddress);
      expect(BigInt(alithAccountBalance)).to.eq(
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

    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
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
        fees: {
          multilocation: [
            {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
          ],
          fungible: transferredBalance / 2n,
        },
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

  const ASSET_MULTILOCATION = {
    parents: 1,
    interior: {
      X3: [
        { Parachain: statemint_para_id },
        { PalletInstance: statemint_assets_pallet_instance },
        { GeneralIndex: 0 },
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

  const assetsToTransfer = 3_300_000_000n * 2n;

  before("should Register an asset and set unit per sec", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await expectOk(context.createBlock(rawTx));

    contractDeployed = contract;

    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
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

    // Deposit asset
    const xcmMessage = new XcmFragment({
      fees: {
        multilocation: [ASSET_MULTILOCATION],
        fungible: assetsToTransfer + 800_000_000n,
      },
      weight_limit: new BN(800_000_000),
      beneficiary: descendOriginAddress,
    })
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
        fees: {
          multilocation: [ASSET_MULTILOCATION],
          fungible: assetsToTransfer / 2n,
        },
        weight_limit: new BN((assetsToTransfer / 2n).toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(2_500_000_000), // 100_000 gas
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
    const { originAddress, descendOriginAddress } = descendOriginFromAddress(context);
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

    const targetXcmWeight = 1_325_000_000n;
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
        fees: {
          multilocation: [
            {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
          ],
          fungible: targetXcmFee,
        },
        weight_limit: new BN(targetXcmWeight.toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(525_000_000), // 100_000 gas
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
    const { originAddress, descendOriginAddress } = descendOriginFromAddress(
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

    const targetXcmWeight = 1_325_000_000n;
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
        fees: {
          multilocation: [
            {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
          ],
          fungible: targetXcmFee,
        },
        weight_limit: new BN(targetXcmWeight.toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(525_000_000), // 100_000 gas
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
    const { originAddress, descendOriginAddress } = descendOriginFromAddress(
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

    const targetXcmWeight = 1_325_000_000n;
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
        fees: {
          multilocation: [
            {
              parents: 0,
              interior: {
                X1: { PalletInstance: balancesPalletIndex },
              },
            },
          ],
          fungible: targetXcmFee,
        },
        weight_limit: new BN(targetXcmWeight.toString()),
        descend_origin: sendingAddress,
      })
        .descend_origin()
        .withdraw_asset()
        .buy_execution()
        .push_any({
          Transact: {
            originType: "SovereignAccount",
            requireWeightAtMost: new BN(525_000_000), // 100_000 gas
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
