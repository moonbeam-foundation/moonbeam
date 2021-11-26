import { expect } from "chai";
import Keyring from "@polkadot/keyring";

import {
  BALTATHAR,
  BALTATHAR_PRIV_KEY,
  ALITH,
  ALITH_PRIV_KEY,
  RANDOM_ADDRESS,
  RANDOM_PRIV_KEY,
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  BOB_AUTHOR_ID,
} from "../../util/constants";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../../util/substrate-rpc";
import {
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
} from "../../util/constants";

const aliceAuthorId = "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
const bobAuthorId = "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";
const charlieAuthorId = "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";

async function getMappingInfo(
  context,
  authorId: string
): Promise<{ account: string; deposit: BigInt }> {
  const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
  return null;
}

describeDevMoonbeam("Proxy : Author Mapping - simple association", (context) => {
  it.only("should succeed in adding an association", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = keyring.addFromUri(ALITH_PRIVATE_KEY, null, "ethereum");
    const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY, null, "ethereum");
    const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY, null, "ethereum");
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    // const { events } = await createBlockWithExtrinsic(
    //   context,
    //   genesisAccount,
    //   context.polkadotApi.tx.authorMapping.addAssociation(bobAuthorId)
    // );

    const { events } = await createBlockWithExtrinsic(
      context,
      alith,
      // @ts-ignore
      context.polkadotApi.tx.proxy.addProxy(baltathar.address, "AuthorMapping", 0)
    );
    expect(events[2].method).to.be.eq("ProxyAdded");
    expect(events[2].data[2].toString()).to.be.eq("AuthorMapping"); //ProxyType
    expect(events[7].method).to.be.eq("ExtrinsicSuccess");
    // expect(events[7].method).to.be.eq("ExtrinsicSuccess");

    const { events: events2 } = await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.proxy.proxy(
        alith.address,
        null,
        context.polkadotApi.tx.authorMapping.addAssociation(BOB_AUTHOR_ID)
      )
    );

    events2.forEach((e) => {
      console.log(2);
      console.log(e.toHuman());
    });
    expect(events2[3].method).to.be.eq("ProxyExecuted");
    console.log("events2[3].data", events2[3].toHuman().data);
    expect(events2[3].data[0].toString()).to.be.eq("Ok");
    expect(events2[6].method).to.be.eq("ExtrinsicSuccess");
    // expect(events2[5].method).to.be.eq("ExtrinsicSuccess");

    // check events
    // expect(events.length === 8);
    // expect(context.polkadotApi.events.balances.Reserved.is(events[1])).to.be.true;
    // expect(context.polkadotApi.events.authorMapping.AuthorRegistered.is(events[2])).to.be.true;
    // expect(context.polkadotApi.events.system.NewAccount.is(events[4])).to.be.true;
    // expect(context.polkadotApi.events.balances.Endowed.is(events[5])).to.be.true;
    // expect(context.polkadotApi.events.treasury.Deposit.is(events[6])).to.be.true;
    // expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[7])).to.be.true;

    // // check association
    expect((await getMappingInfo(context, BOB_AUTHOR_ID)).account).to.eq(ALITH);
    console.log(
      "await context.polkadotApi.query.system.account(ALITH))",
      (await context.polkadotApi.query.system.account(ALITH)).toHuman()
    );
    // expect((await context.polkadotApi.query.system.account(ALITH)).data.free.toBigInt()).to.eq(
    //   1207725818354628455674176n
    // );
    // expect((await context.polkadotApi.query.system.account(ALITH)).data.reserved.toBigInt()).to.eq(
    //   2n * DEFAULT_GENESIS_MAPPING + DEFAULT_GENESIS_STAKING
    // );
  });
});
