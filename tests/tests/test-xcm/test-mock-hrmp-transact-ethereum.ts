import "@moonbeam-network/api-augment";

import { KeyringPair } from "@polkadot/keyring/types";
import { XcmpMessageFormat } from "@polkadot/types/interfaces";
import { BN, u8aToHex } from "@polkadot/util";
import { expect } from "chai";

import { generateKeyringPair } from "../../util/accounts";
import { customWeb3Request } from "../../util/providers";
import { descendOriginFromAllOnes } from "../../util/xcm";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import type { XcmVersionedXcm } from "@polkadot/types/lookup";

import { createContract } from "../../util/transactions";

import { expectOk } from "../../util/expect";


describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM", (context) => {
    let transferredBalance;
    let sendingAddress;
    let random: KeyringPair;
  
    before("Should receive transact action with DescendOrigin", async function () {
      const { originAddress, descendOriginAddress } = descendOriginFromAllOnes(context);
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
  
    it("Should receive transact and should be able to execute ", async function () {
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
        },
        {
            V2: {
            gas_limit: 21000,
            action: {
                Call: random.address,
            },
            value: transferredBalance / 10n,
            input: [],
            access_list: null,
            },
        }
      ];

      let expectedBalance = 0n;

      for(const xcmTransaction of xcmTransactions) {
        expectedBalance += transferredBalance / 10n;
        // TODO need to update lookup type for V2
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
                    fun: { Fungible: transferredBalance / 4n },
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
                    fun: { Fungible: transferredBalance / 4n },
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
    
        expect(testAccountBalance).to.eq(expectedBalance);
      }
    });
  });
  
  describeDevMoonbeam("Mock XCM - receive horizontal transact ETHEREUM", (context) => {
    let transferredBalance;
    let sendingAddress;
    let random: KeyringPair;
    let contractDeployed;
  
    before("Should receive transact action with DescendOrigin and deploy", async function () {
      const { contract, rawTx } = await createContract(context, "Incrementor");
      await expectOk(context.createBlock(rawTx));
  
      contractDeployed = contract;

      const { originAddress, descendOriginAddress } = descendOriginFromAllOnes(context);
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
  
    it("Should receive transact and should be able to execute ", async function () {
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
          }
      ];

      let expectedCalls = 0n;

      for(const xcmTransaction of xcmTransactions) {
        expectedCalls++;
  
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