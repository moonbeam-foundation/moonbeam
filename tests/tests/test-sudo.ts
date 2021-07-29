import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Event } from "@polkadot/types/interfaces";
import {
  GENESIS_ACCOUNT,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  ZERO_ADDRESS,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

describeDevMoonbeam("Sudo - Only sudo account", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it.skip("should NOT be able to call sudo with another account than sudo account", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT))
      .signAndSend(genesisAccount);
    await context.createBlock();
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.toHuman()["account"]).to.equal(ZERO_ADDRESS);
    expect(parachainBondInfo.toHuman()["percent"]).to.equal("30.00%");
  });
  it.skip("should check events", async function () {
    const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events: Event[] = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => event);

      switch (index) {
        // First 3 events are not interesting here
        case 0:
        case 1:
        case 2:
          break;
        // Fourth event
        case 3:
          expect(section === "sudo" && method === "sudo").to.be.true;
          expect(events.length === 4);
          expect(context.polkadotApi.events.system.NewAccount.is(events[0])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[1])).to.be.true;
          expect(context.polkadotApi.events.treasury.Deposit.is(events[2])).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicFailed.is(events[3])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
  });
});
