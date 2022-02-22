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
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";

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

describeDevMoonbeam("Author Mapping - simple association", (context) => {
  it("should match genesis state", async function () {
    expect((await getMappingInfo(context, aliceAuthorId)).account).to.eq(ALITH);
    expect((await getMappingInfo(context, aliceAuthorId)).deposit).to.eq(DEFAULT_GENESIS_MAPPING);
    expect(await getMappingInfo(context, bobAuthorId)).to.eq(null);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.free.toBigInt()
    ).to.eq(1207825819614629174706176n);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.reserved.toBigInt()
    ).to.eq(DEFAULT_GENESIS_MAPPING + DEFAULT_GENESIS_STAKING);
  });

  it("should succeed in adding an association", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const { events } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.authorMapping.addAssociation(bobAuthorId)
    );
    // check events
    expect(events.length === 8);
    expect(context.polkadotApi.events.balances.Reserved.is(events[1] as any)).to.be.true;
    expect(context.polkadotApi.events.authorMapping.AuthorRegistered.is(events[2] as any)).to.be
      .true;
    expect(context.polkadotApi.events.system.NewAccount.is(events[4] as any)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[5] as any)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[6] as any)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[7] as any)).to.be.true;

    // check association
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.free.toBigInt()
    ).to.eq(1207725818354628455674176n);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.reserved.toBigInt()
    ).to.eq(2n * DEFAULT_GENESIS_MAPPING + DEFAULT_GENESIS_STAKING);
  });
});

describeDevMoonbeam("Author Mapping - Fail to reassociate alice", (context) => {
  it("should fail in adding an association for a second time", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
    const { events } = await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.authorMapping.addAssociation(aliceAuthorId)
    );

    // should check events for failure
    expect(events.length === 6);
    expect(context.polkadotApi.events.system.NewAccount.is(events[2] as any)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[3] as any)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4] as any)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[5] as any)).to.be.true;

    //check state
    expect(
      ((await context.polkadotApi.query.system.account(BALTATHAR)) as any).data.free.toBigInt()
    ).to.eq(1208925818354628455674176n);
    expect(
      ((await context.polkadotApi.query.system.account(BALTATHAR)) as any).data.reserved.toBigInt()
    ).to.eq(0n);
    expect((await getMappingInfo(context, aliceAuthorId)).account).to.eq(ALITH);
  });

  it("should fail to take someone else association", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.authorMapping.addAssociation(charlieAuthorId)
    );
    const { events } = await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.authorMapping.updateAssociation(charlieAuthorId, aliceAuthorId)
    );

    // should check events for failure
    expect(
      events.find((e) => e.section == "system" && e.method == "ExtrinsicFailed"),
      "ExtrinsicFailed is missing"
    ).to.not.be.undefined;

    //check state
    expect((await getMappingInfo(context, aliceAuthorId)).account).to.eq(ALITH);
  });
});

describeDevMoonbeam("Author Mapping - Fail without deposit", (context) => {
  before("setup association", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const rando = await keyring.addFromUri(RANDOM_PRIV_KEY, null, "ethereum");
    expect(
      ((await context.polkadotApi.query.system.account(RANDOM_ADDRESS)) as any).data.free.toBigInt()
    ).to.eq(0n);
    try {
      await context.polkadotApi.tx.authorMapping.addAssociation(bobAuthorId).signAndSend(rando);
    } catch (e) {
      expect(e.message.toString()).to.eq(
        "-32000: Invalid transaction validity: Inability to pay some fees " +
          "(e.g. account balance too low)"
      );
    }
    await context.createBlock();
  });

  it("should not add the association", async function () {
    expect(await getMappingInfo(context, bobAuthorId)).to.eq(null);
  });

  // TODO: Fix this test as there is no failed extrinsic in the block
  it.skip("should check events for failure", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    const allRecords = (await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    )) as any;

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => event);

      switch (index) {
        // First 3 events:
        // timestamp.set:: system.ExtrinsicSuccess
        // parachainUpgrade.setValidationData:: system.ExtrinsicSuccess
        // authorInherent.setAuthor:: system.ExtrinsicSuccess
        case 0:
        case 1:
        case 2:
          expect(
            events.length === 1 &&
              context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0] as any)
          ).to.be.true;
          break;
        // Fourth extrinsic
        case 3:
          expect(section === "authorMapping" && method === "addAssociation").to.be.true;
          expect(events.length === 6);
          expect(context.polkadotApi.events.system.NewAccount.is(events[2] as any)).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[3] as any)).to.be.true;
          expect(context.polkadotApi.events.treasury.Deposit.is(events[4] as any)).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[5] as any)).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
    expect(signedBlock.block.extrinsics).to.be.lengthOf(4);
  });
});

describeDevMoonbeam("Author Mapping - double registration", (context) => {
  it("should succeed in adding an association for bob", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    await context.polkadotApi.tx.authorMapping
      .addAssociation(bobAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.free.toBigInt()
    ).to.eq(1207725818354628455674176n);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.reserved.toBigInt()
    ).to.eq(2n * DEFAULT_GENESIS_MAPPING + DEFAULT_GENESIS_STAKING);
  });

  it("should associate with charlie, although already associated with bob", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    await context.polkadotApi.tx.authorMapping
      .addAssociation(charlieAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    //check that both are registered
    expect((await getMappingInfo(context, charlieAuthorId)).account).to.eq(ALITH);
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.free.toBigInt()
    ).to.eq(1207625817094627736646598n);
    expect(
      ((await context.polkadotApi.query.system.account(ALITH)) as any).data.reserved.toBigInt()
    ).to.eq(3n * DEFAULT_GENESIS_MAPPING + DEFAULT_GENESIS_STAKING);
  });
});

describeDevMoonbeam("Author Mapping - registered author can clear (de register)", (context) => {
  it("should succeed in clearing an association", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    await context.polkadotApi.tx.authorMapping
      .addAssociation(bobAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);

    const { events } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.authorMapping.clearAssociation(bobAuthorId)
    );
    //check events
    expect(events.length === 6);
    expect(context.polkadotApi.events.balances.Unreserved.is(events[1] as any)).to.be.true;
    expect(context.polkadotApi.events.authorMapping.AuthorDeRegistered.is(events[2] as any)).to.be
      .true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4] as any)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[5] as any)).to.be.true;

    // check mapping
    expect(await getMappingInfo(context, bobAuthorId)).to.eq(null);
  });
});

describeDevMoonbeam("Author Mapping - unregistered author cannot clear association", (context) => {
  it("should not succeed in clearing an association for an unregistered author", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    expect(await getMappingInfo(context, bobAuthorId)).to.eq(null);

    const { events } = await createBlockWithExtrinsic(
      context,
      genesisAccount,
      context.polkadotApi.tx.authorMapping.clearAssociation(bobAuthorId)
    );
    expect(events.length === 6);
    expect(context.polkadotApi.events.system.NewAccount.is(events[2] as any)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[3] as any)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4] as any)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[5] as any)).to.be.true;
  });
});

describeDevMoonbeam("Author Mapping - non author clearing", (context) => {
  it("should not succeed in clearing an association for a non-author", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    const baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");
    await context.polkadotApi.tx.authorMapping
      .addAssociation(bobAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);

    const { events } = await createBlockWithExtrinsic(
      context,
      baltathar,
      context.polkadotApi.tx.authorMapping.clearAssociation(bobAuthorId)
    );

    expect(events.length === 4);
    expect(context.polkadotApi.events.treasury.Deposit.is(events[2] as any)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[3] as any)).to.be.true;
  });
});

describeDevMoonbeam("Author Mapping - registered can rotate", (context) => {
  it("should succeed in rotating account ids for an author", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    await context.polkadotApi.tx.authorMapping
      .addAssociation(bobAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);
    await context.polkadotApi.tx.authorMapping
      .updateAssociation(bobAuthorId, charlieAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect(await getMappingInfo(context, bobAuthorId)).to.eq(null);
    expect((await getMappingInfo(context, charlieAuthorId)).account).to.eq(ALITH);

    await context.createBlock();
  });
});

describeDevMoonbeam("Author Mapping - unregistered cannot rotate", (context) => {
  it("should fail rotating account ids if not registered", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");

    await context.polkadotApi.tx.authorMapping
      .updateAssociation(bobAuthorId, charlieAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect(await getMappingInfo(context, bobAuthorId)).to.eq(null);
    expect(await getMappingInfo(context, charlieAuthorId)).to.eq(null);

    await context.createBlock();
  });
});

describeDevMoonbeam("Author Mapping - non-author cannot rotate", (context) => {
  it("should fail rotating account ids if not an author", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const baltathar = await keyring.addFromUri(BALTATHAR_PRIV_KEY, null, "ethereum");

    await context.polkadotApi.tx.authorMapping
      .addAssociation(bobAuthorId)
      .signAndSend(genesisAccount);
    await context.createBlock();
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);
    await context.polkadotApi.tx.authorMapping
      .updateAssociation(bobAuthorId, charlieAuthorId)
      .signAndSend(baltathar);
    await context.createBlock();
    expect((await getMappingInfo(context, bobAuthorId)).account).to.eq(ALITH);
    expect(await getMappingInfo(context, charlieAuthorId)).to.eq(null);

    await context.createBlock();
  });
});
