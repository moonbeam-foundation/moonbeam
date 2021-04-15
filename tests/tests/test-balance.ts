import { expect } from "chai";
import { Keyring } from "@polkadot/keyring";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_ACCOUNT,
} from "./constants";
import { Event } from "@polkadot/types/interfaces";
const TEST_ACCOUNT_2 = "0x1111111111111111111111111111111111111112";

describeWithMoonbeam("Moonbeam RPC (Balance)", `simple-specs.json`, (context) => {
  before(async function () {
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: TEST_ACCOUNT,
        value: "0x200", // Must be higher than ExistentialDeposit (0)
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);

    //Inserting polkadot transfer in the second block
    const keyring = new Keyring({ type: "ethereum" });
    const testAccount = await keyring.createFromUri(GENESIS_ACCOUNT_PRIVATE_KEY);
    await context.polkadotApi.tx.balances.transfer(TEST_ACCOUNT_2, 123).signAndSend(testAccount);

    await createAndFinalizeBlock(context.polkadotApi);
  });

  it("genesis balance is setup correctly (web3)", async function () {
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 0)).to.equal(
      GENESIS_ACCOUNT_BALANCE.toString()
    );
  });

  it("genesis balance is setup correctly (polkadotJs)", async function () {
    const genesisHash = await context.polkadotApi.rpc.chain.getBlockHash(0);
    const account = await context.polkadotApi.query.system.account.at(genesisHash, GENESIS_ACCOUNT);
    expect(account.data.free.toString()).to.equal(GENESIS_ACCOUNT_BALANCE.toString());
  });
  it("balance to be updated after transfer", async function () {
    const genesisBalance = BigInt(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 0));
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (genesisBalance - 0x200n - 21000n).toString()
    );
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT, 1)).to.equal("512");
  });

  it("read ethereum.transact extrinsic events", async function () {
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

  it("balance should be the same on polkadot/web3", async function () {
    const block1Hash = await context.polkadotApi.rpc.chain.getBlockHash(1);
    expect(await context.web3.eth.getBalance(GENESIS_ACCOUNT, 1)).to.equal(
      (
        await context.polkadotApi.query.system.account.at(block1Hash, GENESIS_ACCOUNT)
      ).data.free.toString()
    );
  });

  it("transfer from polkadotjs should appear in ethereum", async function () {
    expect(await context.web3.eth.getBalance(TEST_ACCOUNT_2, 2)).to.equal("123");
  });
});
