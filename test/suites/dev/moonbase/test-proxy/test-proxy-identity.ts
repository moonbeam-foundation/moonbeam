import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { GLMR, type KeyringPair, alith, generateKeyringPair } from "@moonwall/util";
import { BN, u8aToU8a } from "@polkadot/util";
import type { EventRecord } from "@polkadot/types/interfaces";
import type { ApiPromise } from "@polkadot/api";
import type { RegistryError } from "@polkadot/types-codec/types";

const getProxyErrors = (api: ApiPromise, events: EventRecord[]): RegistryError[] => {
  return events
    .filter(({ event }) => api.events.proxy.ProxyExecuted.is(event))
    .map(({ event }) => {
      const module = event.data["result"].toJSON()["err"]["module"];
      return api.registry.findMetaError({
        index: new BN(module.index),
        error: u8aToU8a(module.error),
      });
    });
};

describeSuite({
  id: "D023001",
  title: "Proxy : IdentityJudgement",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let identityHash: `0x${string}`;
    let signer: KeyringPair;

    beforeEach(async () => {
      signer = generateKeyringPair("ethereum");

      await context.createBlock(
        context.polkadotJs().tx.balances.transferAllowDeath(signer.address, 5n * GLMR)
      );

      const identityData = {
        display: { Raw: "foobar" },
      };
      const identity = context
        .polkadotJs()
        .registry.createType("PalletIdentityLegacyIdentityInfo", identityData);
      identityHash = identity.hash.toHex();

      const block = await context.createBlock([
        context
          .polkadotJs()
          .tx.sudo.sudo(context.polkadotJs().tx.identity.addRegistrar(alith.address)),
        context.polkadotJs().tx.identity.setIdentity(identityData).signAsync(signer),
      ]);

      block.result?.forEach((r, idx) => {
        expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
      });

      const identityOf = await context.polkadotJs().query.identity.identityOf(signer.address);
      expect(identityOf.unwrap().info.hash.toHex(), "Identity hash should match").to.equal(
        identityHash
      );
    });

    it({
      id: "T01",
      title: "should fail providing judgement",
      test: async () => {
        const blockExecute = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              alith.address,
              null,
              context.polkadotJs().tx.identity.provideJudgement(
                0,
                signer.address,
                {
                  Reasonable: true,
                },
                identityHash
              )
            )
            .signAsync(signer)
        );

        expect(blockExecute.result?.successful).to.be.false;
        expect(blockExecute.result?.error?.name).to.equal("NotProxy");
      },
    });

    it({
      id: "T02",
      title: "should succeed providing judgement",
      test: async () => {
        const blockAdd = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.addProxy(signer.address, "IdentityJudgement", 0)
            .signAsync(alith)
        );

        expect(blockAdd.result?.successful).to.be.true;
        const proxyAddEvent = blockAdd.result?.events.reduce((acc, e) => {
          if (context.polkadotJs().events.proxy.ProxyAdded.is(e.event)) {
            acc.push({
              proxyType: e.event.data[2].toString(),
            });
          }
          return acc;
        }, []);
        expect(proxyAddEvent).to.deep.equal([
          {
            proxyType: "IdentityJudgement",
          },
        ]);

        const blockExecute = await context.createBlock(
          context
            .polkadotJs()
            .tx.proxy.proxy(
              alith.address,
              null,
              context.polkadotJs().tx.identity.provideJudgement(
                0,
                signer.address,
                {
                  Reasonable: true,
                },
                identityHash
              )
            )
            .signAsync(signer)
        );

        expect(blockExecute.result?.successful).to.be.true;
        const proxyExecuteEvent = blockExecute.result?.events.reduce(
          (acc, e) => {
            if (context.polkadotJs().events.proxy.ProxyExecuted.is(e.event)) {
              acc.proxyExecuted = e.event.data[0].toString();
            } else if (context.polkadotJs().events.identity.JudgementGiven.is(e.event)) {
              acc.judgementGiven = {
                address: e.event.data[0].toString(),
                decision: e.event.data[1].toString(),
              };
            }
            return acc;
          },
          { proxyExecuted: null, judgementGiven: null }
        );
        expect(proxyExecuteEvent).to.deep.equal({
          proxyExecuted: "Ok",
          judgementGiven: {
            address: signer.address,
            decision: "0",
          },
        });
      },
    });

    it({
      id: "T03",
      title: "Should fail when calling pallet_identity through a `NonTransfer` proxy",
      test: async () => {
        // Add Alith as NonTransfer Proxy of another account
        const blockAdd = await context.createBlock(
          context.polkadotJs().tx.proxy.addProxy(alith.address, "NonTransfer", 0).signAsync(signer)
        );
        expect(blockAdd.result?.successful).to.be.true;

        let alithNonce = await context
          .viem()
          .getTransactionCount({ address: alith.address as `0x${string}` });
        const blockExecute = await context.createBlock([
          // Alith adds itself as sub account of another account using a proxy call,
          // and reserves a deposit
          await context
            .polkadotJs()
            .tx.proxy.proxy(
              signer.address,
              "NonTransfer",
              context.polkadotJs().tx.identity.addSub(alith.address, { Raw: "test" })
            )
            .signAsync(alith, { nonce: alithNonce++ }),
          // Another flavour of the call above, it does exactly the same thing.
          await context
            .polkadotJs()
            .tx.proxy.proxy(
              signer.address,
              "NonTransfer",
              context.polkadotJs().tx.identity.setSubs([[alith.address, { Raw: "test" }]])
            )
            .signAsync(alith, { nonce: alithNonce++ }),
        ]);
        expect(blockExecute.result!.length).to.equal(2);
        expect(blockExecute.result!.every(({ successful }) => successful)).to.be.true;
        const errors = blockExecute.result!.flatMap(({ events }) =>
          getProxyErrors(context.polkadotJs(), events)
        );
        expect(errors.length).to.equal(2);
        // We expect the calls to fail, `ProxyType` filters these calls
        // for `NonTransfer` proxy calls.
        for (const error of errors) {
          expect(error.docs[0]).to.equal("The origin filter prevent the call to be dispatched.");
          expect(error.name).to.equal("CallFiltered");
        }
      },
    });
  },
});

// describeDevMoonbeam("Proxy : IdentityJudgement succeeds with proxy", (context) => {
//   let identityHash;
//   before("setup one identity and registrar", async () => {
//     const identityData = {
//       display: { Raw: "foobar" },
//     };
//     const identity = context
//       .polkadotJs()
//       .registry.createType("PalletIdentityLegacyIdentityInfo", identityData);
//     identityHash = identity.hash.toHex();
//     const block = await context.createBlock([
//       context
//         .polkadotJs()
//         .tx.sudo.sudo(context.polkadotJs().tx.identity.addRegistrar(alith.address)),
//       context.polkadotJs().tx.identity.setIdentity(identityData).signAsync(baltathar),
//     ]);

//     block.result.forEach((r, idx) => {
//       expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
//     });

//     const identityOf = await context.polkadotJs().query.identity.identityOf(baltathar.address);
//     expect(identityOf.unwrap().info.hash.toHex(), "Identity hash should match").to.equal(
//       identityHash
//     );
//   });

//   it("should succeed providing judgement", async () => {
//     const blockAdd = await context.createBlock(
//       context
//         .polkadotJs()
//         .tx.proxy.addProxy(baltathar.address, "IdentityJudgement" as any, 0)
//         .signAsync(alith)
//     );

//     expect(blockAdd.result.successful).to.be.true;
//     const proxyAddEvent = blockAdd.result.events.reduce((acc, e) => {
//       if (context.polkadotJs().events.proxy.ProxyAdded.is(e.event)) {
//         acc.push({
//           proxyType: e.event.data[2].toString(),
//         });
//       }
//       return acc;
//     }, []);
//     expect(proxyAddEvent).to.deep.equal([
//       {
//         proxyType: "IdentityJudgement",
//       },
//     ]);

//     const blockExecute = await context.createBlock(
//       context
//         .polkadotJs()
//         .tx.proxy.proxy(
//           alith.address,
//           null,
//           // TODO: Remove any casting when api-augment is updated
//           (context.polkadotJs().tx.identity as any).provideJudgement(
//             0,
//             baltathar.address,
//             {
//               Reasonable: true,
//             },
//             identityHash
//           )
//         )
//         .signAsync(baltathar)
//     );

//     expect(blockExecute.result.successful).to.be.true;
//     const proxyExecuteEvent = blockExecute.result.events.reduce(
//       (acc, e) => {
//         if (context.polkadotJs().events.proxy.ProxyExecuted.is(e.event)) {
//           acc.proxyExecuted = e.event.data[0].toString();
//         } else if (context.polkadotJs().events.identity.JudgementGiven.is(e.event)) {
//           acc.judgementGiven = {
//             address: e.event.data[0].toString(),
//             decision: e.event.data[1].toString(),
//           };
//         }
//         return acc;
//       },
//       { proxyExecuted: null, judgementGiven: null }
//     );
//     expect(proxyExecuteEvent).to.deep.equal({
//       proxyExecuted: "Ok",
//       judgementGiven: {
//         address: baltathar.address,
//         decision: "0",
//       },
//     });
//   });
// });
