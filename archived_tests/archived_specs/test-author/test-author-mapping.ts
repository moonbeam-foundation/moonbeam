import "@moonbeam-network/api-augment";

import { expect } from "chai";

import {
  alith,
  ALITH_SESSION_ADDRESS,
  baltathar,
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_SESSION_ADDRESS,
  generateKeyringPair,
} from "../../../util/accounts";
import { DEFAULT_GENESIS_BALANCE, DEFAULT_GENESIS_MAPPING, GLMR } from "../../../util/constants";
import { describeDevMoonbeam, DevTestContext } from "../../../util/setup-dev-tests";

async function getMappingInfo(
  context: DevTestContext,
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
    expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS)).account).to.eq(alith.address);
    expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS)).deposit).to.eq(
      DEFAULT_GENESIS_MAPPING
    );
    expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.free.toBigInt()
    ).to.eq(DEFAULT_GENESIS_BALANCE - DEFAULT_GENESIS_MAPPING);
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.reserved.toBigInt()
    ).to.eq(DEFAULT_GENESIS_MAPPING);
  });

  it("should succeed in adding an association", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    // check events
    expect(events.length === 8);
    expect(context.polkadotApi.events.balances.Reserved.is(events[1].event)).to.be.true;
    expect(context.polkadotApi.events.authorMapping.KeysRegistered.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.system.NewAccount.is(events[4].event)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[5].event)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[6].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[8].event)).to.be.true;

    // check association
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.free.toBigInt() / GLMR
    ).to.eq((DEFAULT_GENESIS_BALANCE - 2n * DEFAULT_GENESIS_MAPPING) / GLMR);
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.reserved.toBigInt()
    ).to.eq(2n * DEFAULT_GENESIS_MAPPING);
  });
});

describeDevMoonbeam("Author Mapping - Fail to reassociate alice", (context) => {
  it("should fail in adding an association for a second time", async function () {
    // Balances before
    const balancesBefore = (
      await context.polkadotApi.query.system.account(baltathar.address)
    ).data.free.toBigInt();

    // Fee
    const fee = (
      await context.polkadotApi.tx.authorMapping
        .addAssociation(ALITH_SESSION_ADDRESS)
        .paymentInfo(baltathar)
    ).partialFee.toBigInt();

    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.authorMapping
        .addAssociation(ALITH_SESSION_ADDRESS)
        .signAsync(baltathar)
    );

    // should check events for failure
    expect(events.length === 6);
    expect(context.polkadotApi.events.system.NewAccount.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[3].event)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[6].event)).to.be.true;

    //check state
    expect(
      (await context.polkadotApi.query.system.account(baltathar.address)).data.free.toBigInt()
    ).to.eq(balancesBefore - fee);
    expect(
      (await context.polkadotApi.query.system.account(baltathar.address)).data.reserved.toBigInt()
    ).to.eq(0n);
    expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS)).account).to.eq(alith.address);
  });

  it("should fail to take someone else association", async function () {
    await context.createBlock(
      context.polkadotApi.tx.authorMapping
        .addAssociation(CHARLETH_SESSION_ADDRESS)
        .signAsync(baltathar)
    );
    const {
      result: { error },
    } = await context.createBlock(
      context.polkadotApi.tx.authorMapping
        .updateAssociation(CHARLETH_SESSION_ADDRESS, ALITH_SESSION_ADDRESS)
        .signAsync(baltathar)
    );

    expect(error.name).to.equal("AlreadyAssociated");

    //check state
    expect((await getMappingInfo(context, ALITH_SESSION_ADDRESS)).account).to.eq(alith.address);
  });
});

describeDevMoonbeam("Author Mapping - Fail without deposit", (context) => {
  before("setup association", async function () {
    const rando = generateKeyringPair();
    expect(
      (await context.polkadotApi.query.system.account(rando.address)).data.free.toBigInt()
    ).to.eq(0n);
    try {
      await context.polkadotApi.tx.authorMapping
        .addAssociation(BALTATHAR_SESSION_ADDRESS)
        .signAndSend(rando);
    } catch (e) {
      expect(e.message.toString()).to.eq(
        "1010: Invalid Transaction: Inability to pay some fees , e.g. account balance too low"
      );
    }
    await context.createBlock();
  });

  it("should not add the association", async function () {
    expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
  });

  // TODO: Fix this test as there is no failed extrinsic in the block
  it.skip("should check events for failure", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events = allRecords.filter(
        ({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index)
      );

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
              context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0].event)
          ).to.be.true;
          break;
        // Fourth extrinsic
        case 3:
          expect(section === "authorMapping" && method === "addAssociation").to.be.true;
          expect(events.length === 6);
          expect(context.polkadotApi.events.system.NewAccount.is(events[2].event)).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[3].event)).to.be.true;
          expect(context.polkadotApi.events.treasury.Deposit.is(events[4].event)).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[5].event)).to.be.true;
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
    // How much fee does it consume the extrinsic
    const fee = (
      await context.polkadotApi.tx.authorMapping
        .addAssociation(BALTATHAR_SESSION_ADDRESS)
        .paymentInfo(alith)
    ).partialFee.toBigInt();

    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);
    const expectedReservecBalance = 2n * DEFAULT_GENESIS_MAPPING;
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.free.toBigInt()
    ).to.eq(DEFAULT_GENESIS_BALANCE - expectedReservecBalance - fee);
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.reserved.toBigInt()
    ).to.eq(expectedReservecBalance);
  });

  it("should associate with charlie, although already associated with bob", async function () {
    // Grab free balance before this test
    let genesisAccountBalanceBefore = (
      await context.polkadotApi.query.system.account(alith.address)
    ).data.free.toBigInt();
    const fee = (
      await context.polkadotApi.tx.authorMapping
        .addAssociation(CHARLETH_SESSION_ADDRESS)
        .paymentInfo(alith)
    ).partialFee.toBigInt();
    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(CHARLETH_SESSION_ADDRESS)
    );
    //check that both are registered
    expect((await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).account).to.eq(alith.address);
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);
    const expectedReservecBalance = 3n * DEFAULT_GENESIS_MAPPING;
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.free.toBigInt()
    ).to.eq(genesisAccountBalanceBefore - DEFAULT_GENESIS_MAPPING - fee);
    expect(
      (await context.polkadotApi.query.system.account(alith.address)).data.reserved.toBigInt()
    ).to.eq(expectedReservecBalance);
  });
});

describeDevMoonbeam("Author Mapping - registered author can clear (de register)", (context) => {
  it("should succeed in clearing an association", async function () {
    await context.createBlock(
      context.polkadotApi.tx.authorMapping
        .addAssociation(BALTATHAR_SESSION_ADDRESS)
        .signAsync(alith)
    );
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);

    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    //check events
    expect(events.length === 6);
    expect(context.polkadotApi.events.balances.Unreserved.is(events[1].event)).to.be.true;
    expect(context.polkadotApi.events.authorMapping.KeysRemoved.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[6].event)).to.be.true;

    // check mapping
    expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
  });
});

describeDevMoonbeam("Author Mapping - unregistered author cannot clear association", (context) => {
  it("should not succeed in clearing an association for an unregistered author", async function () {
    expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);

    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.authorMapping.clearAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    expect(events.length === 6);
    expect(context.polkadotApi.events.system.NewAccount.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.balances.Endowed.is(events[3].event)).to.be.true;
    expect(context.polkadotApi.events.treasury.Deposit.is(events[4].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[6].event)).to.be.true;
  });
});

describeDevMoonbeam("Author Mapping - non author clearing", (context) => {
  it("should not succeed in clearing an association for a non-author", async function () {
    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);

    const {
      result: { events },
    } = await context.createBlock(
      context.polkadotApi.tx.authorMapping
        .clearAssociation(BALTATHAR_SESSION_ADDRESS)
        .signAsync(baltathar)
    );

    expect(events.length === 4);
    expect(context.polkadotApi.events.treasury.Deposit.is(events[2].event)).to.be.true;
    expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[4].event)).to.be.true;
  });
});

describeDevMoonbeam("Author Mapping - registered can rotate", (context) => {
  it("should succeed in rotating account ids for an author", async function () {
    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);

    await context.createBlock(
      context.polkadotApi.tx.authorMapping.updateAssociation(
        BALTATHAR_SESSION_ADDRESS,
        CHARLETH_SESSION_ADDRESS
      )
    );
    expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
    expect((await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).account).to.eq(alith.address);

    await context.createBlock();
  });
});

describeDevMoonbeam("Author Mapping - unregistered cannot rotate", (context) => {
  it("should fail rotating account ids if not registered", async function () {
    await context.createBlock(
      context.polkadotApi.tx.authorMapping.updateAssociation(
        BALTATHAR_SESSION_ADDRESS,
        CHARLETH_SESSION_ADDRESS
      )
    );
    expect(await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).to.eq(null);
    expect(await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).to.eq(null);

    await context.createBlock();
  });
});

describeDevMoonbeam("Author Mapping - non-author cannot rotate", (context) => {
  it("should fail rotating account ids if not an author", async function () {
    await context.createBlock(
      context.polkadotApi.tx.authorMapping.addAssociation(BALTATHAR_SESSION_ADDRESS)
    );
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);

    await context.createBlock(
      context.polkadotApi.tx.authorMapping
        .updateAssociation(BALTATHAR_SESSION_ADDRESS, CHARLETH_SESSION_ADDRESS)
        .signAsync(baltathar)
    );
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(alith.address);
    expect(await getMappingInfo(context, CHARLETH_SESSION_ADDRESS)).to.eq(null);

    await context.createBlock();
  });
});
