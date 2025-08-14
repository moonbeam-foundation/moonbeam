import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_SESSION_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_SESSION_ADDRESS,
  baltathar,
} from "@moonwall/util";
import type { ApiPromise } from "@polkadot/api";
import { getMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D020202",
  title: "Author Mapping - Fail to reassociate alice",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let api: ApiPromise;

    beforeAll(async () => {
      api = context.polkadotJs();
    });

    it({
      id: "T01",
      title: "should fail in adding an association for a second time",
      test: async function () {
        // Balances before
        const balancesBefore = (
          await api.query.system.account(BALTATHAR_ADDRESS)
        ).data.free.toBigInt();

        // Fee
        const fee = (
          await api.tx.authorMapping.addAssociation(ALITH_SESSION_ADDRESS).paymentInfo(baltathar)
        ).partialFee.toBigInt();

        const { result } = await context.createBlock(
          api.tx.authorMapping.addAssociation(ALITH_SESSION_ADDRESS).signAsync(baltathar),
          { allowFailures: true }
        );

        // should check events for failure
        expect(result?.events.length === 6);
        expect(api.events.system.NewAccount.is(result?.events[2].event)).to.be.true;
        expect(api.events.balances.Endowed.is(result?.events[3].event)).to.be.true;
        expect(api.events.system.ExtrinsicFailed.is(result?.events[6].event)).to.be.true;

        //check state
        expect((await api.query.system.account(BALTATHAR_ADDRESS)).data.free.toBigInt()).to.eq(
          balancesBefore - fee
        );
        expect((await api.query.system.account(BALTATHAR_ADDRESS)).data.reserved.toBigInt()).to.eq(
          0n
        );
        expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS))!.account).to.eq(
          ALITH_ADDRESS
        );
      },
    });

    it({
      id: "T02",
      title: "should fail to take someone else association",
      test: async function () {
        await context.createBlock(
          api.tx.authorMapping.addAssociation(CHARLETH_SESSION_ADDRESS).signAsync(baltathar)
        );
        const { result } = await context.createBlock(
          api.tx.authorMapping
            .updateAssociation(CHARLETH_SESSION_ADDRESS, ALITH_SESSION_ADDRESS)
            .signAsync(baltathar),
          { allowFailures: true }
        );

        expect(result!.error!.name).to.equal("AlreadyAssociated");

        //check state
        expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS))!.account).to.eq(
          ALITH_ADDRESS
        );
      },
    });
  },
});
