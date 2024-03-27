import "@moonbeam-network/api-augment";
import { beforeEach, describeSuite, expect } from "@moonwall/cli";
import { GLMR, KeyringPair, alith, generateKeyringPair } from "@moonwall/util";

describeSuite({
  id: "D013005",
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
      expect(identityOf.unwrap()[0].info.hash.toHex(), "Identity hash should match").to.equal(
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
