import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  BOB_AUTHOR_ID,
} from "../../util/constants";
import { getMappingInfo } from "./test-proxy-author-mapping";
import { expectBalanceDifference } from "../../util/balances";
import { substrateTransaction } from "../../util/transactions";
const debug = require("debug")("test:proxy");

// In these tests Alith will allow Baltathar to perform calls on her behalf.
// Charleth is used as a target account when making transfers.
const keyring = new Keyring({ type: "ethereum" });
const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");
const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY, null, "ethereum");

describeDevMoonbeam("Proxy: Balances - should accept known proxy", (context) => {
  it("should accept known proxy", async () => {
    await expectBalanceDifference(
      context,
      CHARLETH_ADDRESS,
      100,
      async () => {
        const events = await substrateTransaction(
          context,
          alith,
          // @ts-ignore //TODO: this is because of https://github.com/polkadot-js/api/issues/4264
          context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Balances", 0)
        );
        expect(events[2].method).to.be.eq("ProxyAdded");
        expect(events[2].data[2].toString()).to.be.eq("Balances"); //ProxyType
        expect(events[7].method).to.be.eq("ExtrinsicSuccess");

        const events2 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.balances.transfer(charleth.address, 100)
          )
        );

        expect(events2[2].method).to.be.eq("ProxyExecuted");
        expect(events2[2].data[0].toString()).to.be.eq("Ok");
        expect(events2[5].method).to.be.eq("ExtrinsicSuccess");
      },
      expect
    );
  });
});

describeDevMoonbeam("Proxy: Balances - shouldn't accept other proxy types", (context) => {
  before("first add proxy", async () => {
    await substrateTransaction(
      context,
      alith,
      // @ts-ignore
      context.polkadotApi.tx.proxy.addProxy(baltathar.address, "Balances", 0)
    );
  });
  it("shouldn't accept other proxy types", async () => {
    await expectBalanceDifference(
      context,
      alith.address,
      0,
      async () => {
        const events2 = await substrateTransaction(
          context,
          baltathar,
          context.polkadotApi.tx.proxy.proxy(
            alith.address,
            null,
            context.polkadotApi.tx.authorMapping.addAssociation(BOB_AUTHOR_ID)
          )
        );

        expect(events2[1].method).to.be.eq("ProxyExecuted");
        expect(events2[1].data[0].toString()).to.be.eq(`{"err":{"badOrigin":null}}`);
        expect(events2[4].method).to.be.eq("ExtrinsicSuccess");

        // // check association failed
        expect(await getMappingInfo(context, BOB_AUTHOR_ID)).to.eq(null);
      },
      expect
    );
  });
});
