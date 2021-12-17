import { expect } from "chai";

import { AnyTuple, IEvent } from "@polkadot/types/types";
import { GENESIS_ACCOUNT } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import Keyring from "@polkadot/keyring";
import { GENESIS_ACCOUNT_PRIVATE_KEY } from "../util/constants";

describeDevMoonbeam("Polkadot API - Header", (context) => {
  it("should return genesis block", async function () {
    const lastHeader = await context.polkadotApi.rpc.chain.getHeader();
    expect(Number(lastHeader.number) >= 0).to.be.true;
  });
});

describeDevMoonbeam("Polkadot API", (context) => {
  before("Setup: Create empty block", async () => {
    await context.createBlock();
  });

  it("should return latest header number", async function () {
    const lastHeader = await context.polkadotApi.rpc.chain.getHeader();
    expect(Number(lastHeader.number)).to.be.at.least(0);
  });

  it("should return latest block number", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    expect(signedBlock.block.header.number.toNumber() >= 0).to.be.true;
  });
});

describeDevMoonbeam("Polkadot API - Transfers", (context) => {
  const testAccount = "0x1111111111111111111111111111111111111111";
  before("Setup: Create empty block with balance.transfer", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    const genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    await context.polkadotApi.tx.balances.transfer(testAccount, 123).signAndSend(genesisAccount);
    await context.createBlock();
  });

  it("should be stored on chain", async function () {
    expect(await context.web3.eth.getBalance(testAccount)).to.equal("123");
  });

  it("should appear in extrinsics", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();

    // Expecting 4 extrinsics so far:
    // timestamp, author, the parachain validation data and the balances transfer.
    expect(signedBlock.block.extrinsics).to.be.of.length(4);

    signedBlock.block.extrinsics.forEach((ex, index) => {
      const {
        method: { args, method, section },
      } = ex;
      const message = `${section}.${method}(${args.map((a) => a.toString()).join(", ")})`;
      switch (index) {
        case 0:
          expect(message.substring(0, 13)).to.eq(`timestamp.set`);
          break;
        case 1:
          expect(message.substring(0, 33)).to.eq(`parachainSystem.setValidationData`);
          break;
        case 2:
          expect(message.substring(0, 42)).to.eq(`authorInherent.kickOffAuthorshipValidation`);
          break;
        case 3:
          expect(message).to.eq(
            `balances.transfer(0x1111111111111111111111111111111111111111, 123)`
          );
          expect(ex.signer.toString()).to.eq(GENESIS_ACCOUNT);
          break;
        default:
          throw new Error(`Unexpected extrinsic: ${message}`);
      }
    });
  });

  it("should appear in events", async function () {
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock();
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

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
          expect(events).to.be.of.length(1);
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0])).to.be.true;
          break;
        // Fourth event: balances.transfer:: system.NewAccount, balances.Endowed, balances.Transfer,
        // system.ExtrinsicSuccess
        case 3:
          events.map((event, index) => {
            console.log(`event[${index}].method: ${event.method}`);
          });
          expect(events).to.be.of.length(9);
          expect(context.polkadotApi.events.system.NewAccount.is(events[1])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[2])).to.be.true;
          expect(context.polkadotApi.events.balances.Transfer.is(events[3])).to.be.true;
          expect(context.polkadotApi.events.system.NewAccount.is(events[5])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[6])).to.be.true;
          expect(context.polkadotApi.events.techCommitteeCollective.Proposed.is(events[7])).to.be;
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[8])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
  });
});
