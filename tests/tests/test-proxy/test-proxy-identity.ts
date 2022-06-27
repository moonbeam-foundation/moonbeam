import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, baltathar } from "../../util/accounts";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Proxy : IdentityJudgement fails without proxy", (context) => {
  before("setup one identity and registrar", async () => {
    const block = await context.createBlock([
      context.polkadotApi.tx.sudo.sudo(context.polkadotApi.tx.identity.addRegistrar(alith.address)),
      context.polkadotApi.tx.identity
        .setIdentity({
          display: { Raw: "foobar" },
        })
        .signAsync(baltathar),
    ]);

    block.result.forEach((r, idx) => {
      expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
    });
  });

  it("should fail providing judgement", async () => {
    const blockExecute = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          alith.address,
          null,
          context.polkadotApi.tx.identity.provideJudgement(0, baltathar.address, {
            Reasonable: true,
          })
        )
        .signAsync(baltathar)
    );

    expect(blockExecute.result.successful).to.be.false;
    expect(blockExecute.result.error.name).to.equal("NotProxy");
  });
});

describeDevMoonbeam("Proxy : IdentityJudgement succeeds with proxy", (context) => {
  before("setup one identity and registrar", async () => {
    const block = await context.createBlock([
      context.polkadotApi.tx.sudo.sudo(context.polkadotApi.tx.identity.addRegistrar(alith.address)),
      context.polkadotApi.tx.identity
        .setIdentity({
          display: { Raw: "foobar" },
        })
        .signAsync(baltathar),
    ]);

    block.result.forEach((r, idx) => {
      expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
    });
  });

  it("should succeed providing judgement", async () => {
    const blockAdd = await context.createBlock(
      context.polkadotApi.tx.proxy
        .addProxy(baltathar.address, "IdentityJudgement" as any, 0)
        .signAsync(alith)
    );

    expect(blockAdd.result.successful).to.be.true;
    const proxyAddEvent = blockAdd.result.events.reduce((acc, e) => {
      if (context.polkadotApi.events.proxy.ProxyAdded.is(e.event)) {
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
      context.polkadotApi.tx.proxy
        .proxy(
          alith.address,
          null,
          context.polkadotApi.tx.identity.provideJudgement(0, baltathar.address, {
            Reasonable: true,
          })
        )
        .signAsync(baltathar)
    );

    expect(blockExecute.result.successful).to.be.true;
    const proxyExecuteEvent = blockExecute.result.events.reduce(
      (acc, e) => {
        if (context.polkadotApi.events.proxy.ProxyExecuted.is(e.event)) {
          acc.proxyExecuted = e.event.data[0].toString();
        } else if (context.polkadotApi.events.identity.JudgementGiven.is(e.event)) {
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
        address: baltathar.address,
        decision: "0",
      },
    });
  });
});
