import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, baltathar, BALTATHAR_SESSION_ADDRESS, ethan } from "../../util/accounts";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { BlockCreationResponse } from "../../util/setup-dev-tests";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
async function expectOk<
  ApiType extends ApiTypes,
  Call extends
    | SubmittableExtrinsic<ApiType>
    | Promise<SubmittableExtrinsic<ApiType>>
    | string
    | Promise<string>,
  Calls extends Call | Call[]
>(
  call: Promise<
    BlockCreationResponse<ApiType, Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>>
  >
) {
  const block = await call;
  if (Array.isArray(block.result)) {
    block.result.forEach((r, idx) => {
      expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
    });
  } else {
    expect(block.result.successful, block.result.error?.name).to.be.true;
  }
}

describeDevMoonbeam("Proxy : IdentityJudgement", (context) => {
  before("setup one identity and one registrar", async () => {
    const block = await context.createBlock([
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.identity.addRegistrar(baltathar.address)
      ),
      context.polkadotApi.tx.identity
        .setIdentity({
          display: { Raw: "foobar" },
        })
        .signAsync(ethan),
    ]);

    block.result.forEach((r, idx) => {
      expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
    });
  });

  it("should succeed providing judgement", async () => {
    const blockAdd = await context.createBlock(
      context.polkadotApi.tx.proxy
        .addProxy(ethan.address, "IdentityJudgement" as any, 0)
        .signAsync(baltathar)
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
          baltathar.address,
          null,
          context.polkadotApi.tx.identity.provideJudgement(0, ethan.address, {
            Reasonable: true,
          })
        )
        .signAsync(ethan)
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
        address: ethan.address,
        decision: "0",
      },
    });
  });
});
