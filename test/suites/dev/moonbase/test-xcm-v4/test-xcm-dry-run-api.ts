import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { alith, generateKeyringPair } from "@moonwall/util";
import { ApiPromise, WsProvider } from "@polkadot/api";
import { u8aToHex } from "@polkadot/util";
import { XcmFragment } from "../../../../helpers";

// TODO: remove once the api is present in @polkadot/api
const runtimeApi = {
  runtime: {
    DryRunApi: [
      {
        methods: {
          dry_run_call: {
            description: "Dry run call",
            params: [
              {
                name: "origin",
                type: "OriginCaller",
              },
              {
                name: "call",
                type: "Call",
              },
            ],
            type: "Result<CallDryRunEffects<Event>, XcmDryRunError>",
          },
          dry_run_xcm: {
            description: "Dry run XCM program",
            params: [
              {
                name: "origin_location",
                type: "XcmVersionedLocation",
              },
              {
                name: "xcm",
                type: "XcmVersionedXcm",
              },
            ],
            type: "Result<XcmDryRunEffects, XcmDryRunError>",
          },
        },
        version: 1,
      },
    ],
  },
  types: {
    CallDryRunEffects: {
      ExecutionResult: "DispatchResultWithPostInfo",
      EmittedEvents: "Vec<Event>",
      LocalXcm: "Option<XcmVersionedXcm>",
      ForwardedXcms: "Vec<(XcmVersionedLocation, Vec<XcmVersionedXcm>)>",
    },
    DispatchErrorWithPostInfoTPostDispatchInfo: {
      postInfo: "PostDispatchInfo",
      error: "DispatchError",
    },
    DispatchResultWithPostInfo: {
      _enum: {
        Ok: "PostDispatchInfo",
        Err: "DispatchErrorWithPostInfoTPostDispatchInfo",
      },
    },
    PostDispatchInfo: {
      actualWeight: "Option<Weight>",
      paysFee: "Pays",
    },
    XcmDryRunEffects: {
      ExecutionResult: "StagingXcmV4TraitsOutcome",
      EmittedEvents: "Vec<Event>",
      ForwardedXcms: "Vec<(XcmVersionedLocation, Vec<XcmVersionedXcm>)>",
    },
    XcmDryRunError: {
      _enum: {
        Unimplemented: "Null",
        VersionedConversionFailed: "Null",
      },
    },
  },
};

describeSuite({
  id: "D014135",
  title: "XCM - DryRunApi",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let polkadotJs: ApiPromise;

    beforeAll(async function () {
      polkadotJs = await ApiPromise.create({
        provider: new WsProvider(`ws://localhost:${process.env.MOONWALL_RPC_PORT}/`),
        ...runtimeApi,
      });
    });

    it({
      id: "T01",
      title: "Should succeed calling DryRunApi::dryRunCall",
      test: async function () {
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const randomReceiver = "0x1111111111111111111111111111111111111111111111111111111111111111";

        // Beneficiary from destination's point of view
        const destBeneficiary = {
          V3: {
            parents: 0,
            interior: {
              X1: {
                AccountId32: {
                  network: null,
                  id: randomReceiver,
                },
              },
            },
          },
        };

        const assetsToSend = {
          V3: [
            {
              id: {
                Concrete: {
                  parents: 0,
                  interior: {
                    X1: { PalletInstance: Number(balancesPalletIndex) },
                  },
                },
              },
              fun: {
                Fungible: 1_000_000_000_000_000n,
              },
            },
          ],
        };
        const dest = {
          V3: {
            parents: 1,
            interior: {
              Here: null,
            },
          },
        };
        const polkadotXcmTx = polkadotJs.tx.polkadotXcm.transferAssets(
          dest,
          destBeneficiary,
          assetsToSend,
          0,
          "Unlimited"
        );

        const dryRunCall = await polkadotJs.call.dryRunApi.dryRunCall(
          { System: { signed: alith.address } },
          polkadotXcmTx
        );

        expect(dryRunCall.isOk).to.be.true;
        expect(dryRunCall.asOk.executionResult.isOk).be.true;
      },
    });

    it({
      id: "T02",
      title: "Should succeed calling DryRunApi::dryRunXcm",
      test: async function () {
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();
        const randomKeyPair = generateKeyringPair();

        // We will dry run a "ReserveAssetDeposited" coming from the relay
        const xcmMessage = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: Number(balancesPalletIndex) },
                },
              },
              fungible: 1_000_000_000_000_000n,
            },
          ],
          beneficiary: u8aToHex(randomKeyPair.addressRaw),
        })
          .reserve_asset_deposited()
          .clear_origin()
          .buy_execution()
          .deposit_asset_v3()
          .as_v3();

        const dryRunXcm = await polkadotJs.call.dryRunApi.dryRunXcm(
          {
            V3: {
              Concrete: { parent: 1, interior: { Here: null } },
            },
          },
          xcmMessage
        );

        expect(dryRunXcm.isOk).to.be.true;
        expect(dryRunXcm.asOk.executionResult.isComplete).be.true;
      },
    });
  },
});
