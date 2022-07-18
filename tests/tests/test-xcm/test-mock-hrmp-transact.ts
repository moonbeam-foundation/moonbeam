import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { XcmpMessageFormat } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";

import { generateKeyringPair } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm } from "@polkadot/types/lookup";

import { createContract } from "../../util/transactions";

import { expectOk } from "../../util/expect";

describeDevMoonbeam("Mock XCM - receive horizontal transact", (context) => {
  let transferredBalance;
  let DescendOriginAddress;
  let sendingAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;
    random = generateKeyringPair();
    const derivedMultiLocation = context.polkadotApi.createType(
      "MultiLocation",
      JSON.parse(
        `{\
              "parents": 1,\
              "interior": {\
                "X2": [\
                  { "Parachain": 1 },\
                  { "AccountKey20": \
                    {\
                      "network": "Any",\
                      "key": "${allones}"\
                    } \
                  }\
                ]\
              }\
            }`
      )
    );

    const toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 1000000000000000000n;

    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
      )
    );
    const balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const transferCall = context.polkadotApi.tx.balances.transfer(
      random.address,
      transferredBalance / 10n
    );
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
            requireWeightAtMost: new BN(1000000000),
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

    expect(testAccountBalance).to.eq(transferredBalance / 10n);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM", (context) => {
  let transferredBalance;
  let DescendOriginAddress;
  let sendingAddress;
  let random: KeyringPair;

  before("Should receive transact action with DescendOrigin", async function () {
    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;
    random = generateKeyringPair();
    const derivedMultiLocation = context.polkadotApi.createType(
      "MultiLocation",
      JSON.parse(
        `{\
              "parents": 1,\
              "interior": {\
                "X2": [\
                  { "Parachain": 1 },\
                  { "AccountKey20": \
                    {\
                      "network": "Any",\
                      "key": "${allones}"\
                    } \
                  }\
                ]\
              }\
            }`
      )
    );

    const toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
      )
    );
    const balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const xcmTransaction = {
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
        value: transferredBalance / 10n,
        input: [],
        access_list: null,
      },
    };

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
            requireWeightAtMost: new BN(2000000000),
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

    expect(testAccountBalance).to.eq(transferredBalance / 10n);
  });
});

describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM", (context) => {
  let transferredBalance;
  let DescendOriginAddress;
  let sendingAddress;
  let random: KeyringPair;
  let contractDeployed;

  before("Should receive transact action with DescendOrigin and deploy", async function () {
    const { contract, rawTx } = await createContract(context, "Incrementor");
    await expectOk(context.createBlock(rawTx));

    contractDeployed = contract;

    const allones = "0x0101010101010101010101010101010101010101";
    sendingAddress = allones;
    random = generateKeyringPair();
    const derivedMultiLocation = context.polkadotApi.createType(
      "MultiLocation",
      JSON.parse(
        `{\
              "parents": 1,\
              "interior": {\
                "X2": [\
                  { "Parachain": 1 },\
                  { "AccountKey20": \
                    {\
                      "network": "Any",\
                      "key": "${allones}"\
                    } \
                  }\
                ]\
              }\
            }`
      )
    );

    const toHash = new Uint8Array([
      ...new Uint8Array([32]),
      ...new TextEncoder().encode("multiloc"),
      ...derivedMultiLocation.toU8a(),
    ]);

    DescendOriginAddress = u8aToHex(context.polkadotApi.registry.hash(toHash).slice(0, 20));

    transferredBalance = 10_000_000_000_000_000_000n;

    // We first fund parachain 2000 sovreign account
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.balances.transfer(DescendOriginAddress, transferredBalance)
      )
    );
    const balance = (
      (await context.polkadotApi.query.system.account(DescendOriginAddress)) as any
    ).data.free.toBigInt();
    expect(balance).to.eq(transferredBalance);
  });

  it("Should receive transact and should be able to execute ", async function () {
    // Get Pallet balances index
    const metadata = await context.polkadotApi.rpc.state.getMetadata();
    const balancesPalletIndex = (metadata.asLatest.toHuman().pallets as Array<any>).find(
      (pallet) => {
        return pallet.name === "Balances";
      }
    ).index;

    const xcmTransaction = {
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
    };

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

    expect(await contractDeployed.methods.count().call()).to.eq("1");
  });
});
