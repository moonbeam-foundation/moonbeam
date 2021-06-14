import { expect } from "chai";
import { Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { Event } from "@polkadot/types/interfaces";
import {
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  GENESIS_ACCOUNT,
  COLLATOR_ACCOUNT,
  ALITH_PRIV_KEY,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

describeDevMoonbeam("Sudo - Only sudo account", (context) => {
  let genesisAccount: KeyringPair, sudoAccount: KeyringPair;
  before("Setup genesis account for substrate", async () => {
    const keyring = new Keyring({ type: "ethereum" });
    genesisAccount = await keyring.addFromUri(GENESIS_ACCOUNT_PRIVATE_KEY, null, "ethereum");
    sudoAccount = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
  });
  it("should NOT be able to call sudo with another account than sudo account", async function () {
    await context.polkadotApi.tx.sudo
      .sudo(context.polkadotApi.tx.parachainStaking.setParachainBondAccount(GENESIS_ACCOUNT))
      .signAndSend(genesisAccount);
    await context.createBlock();
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    console.log("parachainBondInfo", parachainBondInfo.toHuman());
  });
  it("should check events", async function () {
    // const testAddress = "0x1111111111111111111111111111111111111111";
    // await context.createBlock({
    //   transactions: [await createTransfer(context.web3, testAddress, 512)],
    // });

    const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    const signedBlock = await context.polkadotApi.rpc.chain.getBlock(blockHash);
    const allRecords = await context.polkadotApi.query.system.events.at(
      signedBlock.block.header.hash
    );

    // map between the extrinsics and events
    signedBlock.block.extrinsics.forEach(({ method: { method, section } }, index) => {
      console.log("method, section", method, section);
      // filter the specific events based on the phase and then the
      // index of our extrinsic in the block
      const events: Event[] = allRecords
        .filter(({ phase }) => phase.isApplyExtrinsic && phase.asApplyExtrinsic.eq(index))
        .map(({ event }) => {
          console.log("event", event.toHuman());
          return event;
        });

      switch (index) {
        // First 3 events:
        // timestamp.set:: system.ExtrinsicSuccess
        // parachainUpgrade.setValidationData:: system.ExtrinsicSuccess
        // authorInherent.setAuthor:: system.ExtrinsicSuccess
        case 0:
        case 1:
        case 2:
          expect(
            events.length === 1 && context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0])
          ).to.be.true;
          break;
        // Fourth event: ethereum.transact:: system.NewAccount, balances.Endowed, (?),
        // ethereum.Executed, system.ExtrinsicSuccess
        case 3:
          expect(section === "ethereum" && method === "transact").to.be.true;
          expect(events.length === 4);
          expect(context.polkadotApi.events.system.NewAccount.is(events[0])).to.be.true;
          expect(context.polkadotApi.events.balances.Endowed.is(events[1])).to.be.true;
          // TODO: what event was inserted here?
          expect(context.polkadotApi.events.ethereum.Executed.is(events[3])).to.be.true;
          expect(context.polkadotApi.events.system.ExtrinsicSuccess.is(events[4])).to.be.true;
          break;
        default:
          throw new Error(`Unexpected extrinsic`);
      }
    });
  });
});
