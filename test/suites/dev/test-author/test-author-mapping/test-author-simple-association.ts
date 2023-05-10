import "@moonbeam-network/api-augment";
import {
  alith,
  ALITH_SESSION_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  DEFAULT_GENESIS_BALANCE,
  DEFAULT_GENESIS_MAPPING,
  GLMR,
} from "@moonwall/util";
import { expect, describeSuite, beforeAll } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../../helpers/common.js";

describeSuite({
  id: "D228",
  title: "Author Mapping - simple association",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;
    it({
      id: "T01",
      title: "should match genesis state",
      test: async function () {
        api = context.polkadotJs({ type: "moon" });
        expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS)).account).to.eq(alith.address);
        expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS)).deposit).to.eq(
          DEFAULT_GENESIS_MAPPING
        );
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
        expect((await api.query.system.account(alith.address)).data.free.toBigInt()).to.eq(
          DEFAULT_GENESIS_BALANCE - DEFAULT_GENESIS_MAPPING
        );
        expect((await api.query.system.account(alith.address)).data.reserved.toBigInt()).to.eq(
          DEFAULT_GENESIS_MAPPING
        );
      },
    });

    it({
      id: "T02",
      title: "should succeed in adding an association",
      test: async function () {
        const {
          result: { events },
        } = await context.createBlock(
          api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
        );
        // check events
        expect(events.length === 8);
        expect(api.events.balances.Reserved.is(events[1].event)).to.be.true;
        expect(api.events.authorMapping.KeysRegistered.is(events[2].event)).to.be.true;
        expect(api.events.system.NewAccount.is(events[4].event)).to.be.true;
        expect(api.events.balances.Endowed.is(events[5].event)).to.be.true;
        expect(api.events.treasury.Deposit.is(events[6].event)).to.be.true;
        expect(api.events.system.ExtrinsicSuccess.is(events[8].event)).to.be.true;

        // check association
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(
          alith.address
        );
        expect((await api.query.system.account(alith.address)).data.free.toBigInt() / GLMR).to.eq(
          (DEFAULT_GENESIS_BALANCE - 2n * DEFAULT_GENESIS_MAPPING) / GLMR
        );
        expect((await api.query.system.account(alith.address)).data.reserved.toBigInt()).to.eq(
          2n * DEFAULT_GENESIS_MAPPING
        );
      },
    });
  },
});
