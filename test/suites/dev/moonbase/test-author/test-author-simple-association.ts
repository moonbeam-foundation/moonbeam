import "@moonbeam-network/api-augment";
import {
  ALITH_ADDRESS,
  ALITH_SESSION_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  DEFAULT_GENESIS_BALANCE,
  DEFAULT_GENESIS_MAPPING,
  GLMR,
} from "@moonwall/util";
import { expect, describeSuite } from "@moonwall/cli";
import type { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020212",
  title: "Author Mapping - simple association",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let api: ApiPromise;
    it({
      id: "T01",
      title: "should match genesis state",
      test: async function () {
        api = context.polkadotJs();
        expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS))?.account).to.eq(
          ALITH_ADDRESS
        );
        expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS))?.deposit).to.eq(
          DEFAULT_GENESIS_MAPPING
        );
        expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).toBeUndefined();
        expect((await api.query.system.account(ALITH_ADDRESS)).data.free.toBigInt()).to.eq(
          DEFAULT_GENESIS_BALANCE - DEFAULT_GENESIS_MAPPING
        );
        expect((await api.query.system.account(ALITH_ADDRESS)).data.reserved.toBigInt()).to.eq(
          DEFAULT_GENESIS_MAPPING
        );
      },
    });

    it({
      id: "T02",
      title: "should succeed in adding an association",
      test: async function () {
        const { result } = await context.createBlock(
          api.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
        );
        // check events
        expect(result?.events.length === 8);
        expect(api.events.balances.Reserved.is(result?.events[1].event)).to.be.true;
        expect(api.events.authorMapping.KeysRegistered.is(result?.events[2].event)).to.be.true;
        expect(api.events.system.NewAccount.is(result?.events[4].event)).to.be.true;
        expect(api.events.balances.Endowed.is(result?.events[5].event)).to.be.true;
        expect(api.events.system.ExtrinsicSuccess.is(result?.events[8].event)).to.be.true;

        // check association
        expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS))?.account).to.eq(
          ALITH_ADDRESS
        );
        expect((await api.query.system.account(ALITH_ADDRESS)).data.free.toBigInt() / GLMR).to.eq(
          (DEFAULT_GENESIS_BALANCE - 2n * DEFAULT_GENESIS_MAPPING) / GLMR
        );
        expect((await api.query.system.account(ALITH_ADDRESS)).data.reserved.toBigInt()).to.eq(
          2n * DEFAULT_GENESIS_MAPPING
        );
      },
    });
  },
});
