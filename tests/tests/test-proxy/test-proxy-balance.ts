import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import { getMappingInfo } from "./test-proxy-author-mapping";
import { expectBalanceDifference } from "../../util/balances";
import { alith, baltathar, BALTATHAR_SESSION_ADDRESS, charleth } from "../../util/accounts";
const debug = require("debug")("test:proxy");

describeDevMoonbeam("Proxy: Balances - should accept known proxy", (context) => {
  it("should accept known proxy", async () => {
    await expectBalanceDifference(
      context,
      charleth.address,
      100,
      async () => {
        const {
          result: { events },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Balances" as any, 0)
        );
        expect(events[2].event.method).to.be.eq("ProxyAdded");
        expect(events[2].event.data[2].toString()).to.be.eq("Balances"); //ProxyType
        expect(events[7].event.method).to.be.eq("ExtrinsicSuccess");

        const {
          result: { events: events2 },
        } = await context.createBlock(
          context.polkadotApi.tx.proxy
            .proxy(
              alith.address,
              null,
              context.polkadotApi.tx.balances.transfer(charleth.address, 100)
            )
            .signAsync(baltathar)
        );

        expect(events2[2].event.method).to.be.eq("ProxyExecuted");
        expect(events2[2].event.data[0].toString()).to.be.eq("Ok");
        expect(events2[5].event.method).to.be.eq("ExtrinsicSuccess");
      },
      expect
    );
  });
});

describeDevMoonbeam("Proxy: Balances - shouldn't accept other proxy types", (context) => {
  before("first add proxy", async () => {
    await context.createBlock(
      context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Balances" as any, 0)
    );
  });
  it("shouldn't accept other proxy types", async () => {
    await expectBalanceDifference(
      context,
      alith.address,
      0,
      async () => {
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

        expect(events2[1].event.method).to.be.eq("ProxyExecuted");
        expect(events2[1].event.data[0].toString()).to.be.eq(
          `{"err":{"module":{"index":0,"error":"0x05000000"}}}`
        );
        expect(events2[4].event.method).to.be.eq("ExtrinsicSuccess");

        // // check association failed
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
      },
      expect
    );
  });
});
