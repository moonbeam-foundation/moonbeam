import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { alith, baltathar, BALTATHAR_SESSION_ADDRESS } from "../../util/accounts";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Proxy : IdentityJudgement - simple association", (context) => {
  it("should succeed in adding an association", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(baltathar.address, "IdentityJudgement" as any, 0)
    );
    expect(events[2].event.method).to.be.eq("ProxyAdded");
    expect(events[2].event.data[2].toString()).to.be.eq("IdentityJudgement"); //ProxyType
    expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");
    const {
      result: { events: events2 },
    } = await context.createBlock(
      context.polkadotApi.tx.proxy
        .proxy(
          alith.address,
          null,
          context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
        )
        .signAsync(baltathar)
    );

    expect(events2[3].event.method).to.be.eq("ProxyExecuted");
    expect(events2[3].event.data[0].toString()).to.be.eq("Ok");
    expect(events2[6].event.method).to.be.eq("ExtrinsicSuccess");

    // // check data
  });
});
