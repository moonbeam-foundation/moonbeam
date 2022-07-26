import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { XcmpMessageFormat } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";

import { generateKeyringPair } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { descendOriginFromAddress, registerForeignAsset } from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm } from "@polkadot/types/lookup";

import { createContract } from "../../util/transactions";

import { expectOk } from "../../util/expect";

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM (transfer)", (context) => {
  let transferredBalance;
  let sendingAddress;
  let descendAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
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

  it("Should receive transact and should be able to execute", async function () {
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

    const targetXcmWeight = 925_000_000n;
    const targetXcmFee = targetXcmWeight * 50_000n;

    for (const xcmTransaction of xcmTransactions) {
      expectedTransferredAmount += amountToTransfer;
      expectedTransferredAmountPlusFees += amountToTransfer + targetXcmFee;
      // TODO need to update lookup types for xcm ethereum transaction V2
      const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction);
      const transferCallEncoded = transferCall?.method.toHex();

      // We are going to test that we can receive a transact operation from parachain 1
      // using descendOrigin first
      const xcmMessage = {
        V2: [
          {
            DescendOrigin: {
              X1: {
                AccountKey20: {
                  network: "Any",
                  key: sendingAddress,
                },
              },
            },
          },
          {
            WithdrawAsset: [
              {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
                    },
                  },
                },
                fun: { Fungible: targetXcmFee },
              },
            ],
          },
          {
            BuyExecution: {
              fees: {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
                    },
                  },
                },
                fun: { Fungible: targetXcmFee },
              },
              weightLimit: { Limited: targetXcmWeight },
            },
          },
          {
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(525_000_000), // 21_000 gas limit
              call: {
                encoded: transferCallEncoded,
              },
            },
          },
        ],
      };
      const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
        "XcmpMessageFormat",
        "ConcatenatedVersionedXcm"
      ) as any;
      const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
        "XcmVersionedXcm",
        xcmMessage
      ) as any;

      const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
      // Send RPC call to inject XCM message
      // We will set a specific message knowing that it should mint the statemint asset
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessage]);

      // Create a block in which the XCM will be executed
      await context.createBlock();

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

  before("Should receive transact action with DescendOrigin and deploy", async function () {
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

  it("Should receive transact and should be able to execute", async function () {
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
      const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction);
      const transferCallEncoded = transferCall?.method.toHex();
      // We are going to test that we can receive a transact operation from parachain 1
      // using descendOrigin first
      const xcmMessage = {
        V2: [
          {
            DescendOrigin: {
              X1: {
                AccountKey20: {
                  network: "Any",
                  key: sendingAddress,
                },
              },
            },
          },
          {
            WithdrawAsset: [
              {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
                    },
                  },
                },
                fun: { Fungible: transferredBalance / 2n },
              },
            ],
          },
          {
            BuyExecution: {
              fees: {
                id: {
                  Concrete: {
                    parents: 0,
                    interior: {
                      X1: { PalletInstance: balancesPalletIndex },
                    },
                  },
                },
                fun: { Fungible: transferredBalance / 2n },
              },
              weightLimit: { Limited: new BN(4000000000) },
            },
          },
          {
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(3000000000),
              call: {
                encoded: transferCallEncoded,
              },
            },
          },
        ],
      };
      const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
        "XcmpMessageFormat",
        "ConcatenatedVersionedXcm"
      ) as any;
      const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
        "XcmVersionedXcm",
        xcmMessage
      ) as any;

      const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
      // Send RPC call to inject XCM message
      // We will set a specific message knowing that it should mint the statemint asset
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessage]);

      // Create a block in which the XCM will be executed
      await context.createBlock();

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

  const assetsToTransfer = 2_900_000_000n * 2n;

  before("Should Register an asset and set unit per sec", async function () {
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
    expect(events[4].event.method.toString()).to.eq("ExtrinsicSuccess");
    expect(registeredAsset.owner.toHex()).to.eq(palletId.toLowerCase());

    // Deposit asset
    let xcmMessage = {
      V2: [
        {
          ReserveAssetDeposited: [
            {
              id: {
                Concrete: ASSET_MULTILOCATION,
              },
              fun: { Fungible: assetsToTransfer + 400_000_000n },
            },
          ],
        },
        { ClearOrigin: null as any },
        {
          BuyExecution: {
            fees: {
              id: {
                Concrete: ASSET_MULTILOCATION,
              },
              fun: { Fungible: assetsToTransfer + 400_000_000n },
            },
            weightLimit: { Limited: new BN(400_000_000) },
          },
        },
        {
          DepositAsset: {
            assets: { Wild: "All" },
            maxAssets: new BN(1),
            beneficiary: {
              parents: 0,
              interior: { X1: { AccountKey20: { network: "Any", key: descendOriginAddress } } },
            },
          },
        },
      ],
    };
    const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
      "XcmpMessageFormat",
      "ConcatenatedVersionedXcm"
    ) as any;
    const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
      "XcmVersionedXcm",
      xcmMessage
    ) as any;

    const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
    // Send RPC call to inject XCM message
    // We will set a specific message knowing that it should mint the statemint asset
    const r = await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [
      statemint_para_id,
      totalMessage,
    ]);
    // Create a block in which the XCM will be executed
    await context.createBlock();
    // Make sure descended address has the transferred foreign assets (minus the xcm fees).
    expect(
      (await context.polkadotApi.query.assets.account(assetId, descendedAddress))
        .unwrap()
        .balance.toBigInt()
    ).to.eq(assetsToTransfer);
  });

  it("Should receive transact and should be able to execute", async function () {
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
      const transferCall = context.polkadotApi.tx.ethereumXcm.transact(xcmTransaction);
      const transferCallEncoded = transferCall?.method.toHex();
      // We are going to test that we can receive a transact operation from parachain 1
      // using descendOrigin first
      const xcmMessage = {
        V2: [
          {
            DescendOrigin: {
              X1: {
                AccountKey20: {
                  network: "Any",
                  key: sendingAddress,
                },
              },
            },
          },
          {
            WithdrawAsset: [
              {
                id: {
                  Concrete: ASSET_MULTILOCATION,
                },
                fun: { Fungible: assetsToTransfer / 2n },
              },
            ],
          },
          {
            BuyExecution: {
              fees: {
                id: {
                  Concrete: ASSET_MULTILOCATION,
                },
                fun: { Fungible: assetsToTransfer / 2n },
              },
              weightLimit: { Limited: assetsToTransfer / 2n },
            },
          },
          {
            Transact: {
              originType: "SovereignAccount",
              requireWeightAtMost: new BN(2_500_000_000), // 100_000 gas
              call: {
                encoded: transferCallEncoded,
              },
            },
          },
        ],
      };
      const xcmpFormat: XcmpMessageFormat = context.polkadotApi.createType(
        "XcmpMessageFormat",
        "ConcatenatedVersionedXcm"
      ) as any;
      const receivedMessage: XcmVersionedXcm = context.polkadotApi.createType(
        "XcmVersionedXcm",
        xcmMessage
      ) as any;

      const totalMessage = [...xcmpFormat.toU8a(), ...receivedMessage.toU8a()];
      // Send RPC call to inject XCM message
      // We will set a specific message knowing that it should mint the statemint asset
      await customWeb3Request(context.web3, "xcm_injectHrmpMessage", [1, totalMessage]);

      // Create a block in which the XCM will be executed
      await context.createBlock();

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
