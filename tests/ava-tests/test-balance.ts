import { test } from "../util/setup";

import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_BALANCE,
  GENESIS_ACCOUNT_PRIVATE_KEY,
} from "../util/constants";
import { Event } from "@polkadot/types/interfaces";
import { EnhancedWeb3 } from "../util/providers";

const sendTransfer = async (
  web3: EnhancedWeb3,
  from: string,
  to: string,
  amount: number | string
) => {
  const tx = await web3.eth.accounts.signTransaction(
    {
      from,
      to,
      value: amount,
      gasPrice: "0x01",
      gas: 21000,
    },
    GENESIS_ACCOUNT_PRIVATE_KEY
  );
  await web3.customRequest("eth_sendRawTransaction", [tx.rawTransaction]);
};

test("Moonbeam RPC (Balance) - genesis balance is setup correctly (web3)", async (t) => {
  t.is(await t.context.web3.eth.getBalance(GENESIS_ACCOUNT, 0), GENESIS_ACCOUNT_BALANCE.toString());
});

test("Moonbeam RPC (Balance) - genesis balance is setup correctly (polkadotJs)", async (t) => {
  const genesisHash = await t.context.polkadotApi.rpc.chain.getBlockHash(0);
  const account = await t.context.polkadotApi.query.system.account.at(genesisHash, GENESIS_ACCOUNT);

  t.is(account.data.free.toString(), GENESIS_ACCOUNT_BALANCE.toString());
});

test("Moonbeam RPC (Balance) - balance to be updated after transfer", async (t) => {
  const testAddress = "0x1111111111111111111111111111111111111111";
  const genesisBalance = BigInt(await t.context.web3.eth.getBalance(GENESIS_ACCOUNT, 0));

  await sendTransfer(t.context.web3, GENESIS_ACCOUNT, testAddress, "0x200");
  await t.context.createAndFinalizeBlock();

  t.is(
    await t.context.web3.eth.getBalance(GENESIS_ACCOUNT, 1),
    (genesisBalance - 0x200n - 21000n).toString()
  );
  t.is(await t.context.web3.eth.getBalance(testAddress, 1), "512");
});

test("Moonbeam RPC (Balance) - balance should be the same on polkadot/web3", async (t) => {
  const testAddress = "0x1111111111111111111111111111111111111111";

  await sendTransfer(t.context.web3, GENESIS_ACCOUNT, testAddress, "0x200");
  await t.context.createAndFinalizeBlock();
  const block1Hash = await t.context.polkadotApi.rpc.chain.getBlockHash(1);
  const web3Balance = await t.context.web3.eth.getBalance(GENESIS_ACCOUNT, 1);
  const polkadotBalance = await t.context.polkadotApi.query.system.account.at(
    block1Hash,
    GENESIS_ACCOUNT
  );

  t.is(web3Balance, polkadotBalance.data.free.toString());
});

test("Moonbeam RPC (Balance) - read ethereum.transact extrinsic events", async (t) => {
  const testAddress = "0x1111111111111111111111111111111111111111";

  await sendTransfer(t.context.web3, GENESIS_ACCOUNT, testAddress, "0x200");
  await t.context.createAndFinalizeBlock();

  const blockHash = await t.context.polkadotApi.rpc.chain.getBlockHash(1);
  const signedBlock = await t.context.polkadotApi.rpc.chain.getBlock(blockHash);
  const allRecords = await t.context.polkadotApi.query.system.events.at(
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
        t.is(events.length, 1);
        t.true(t.context.polkadotApi.events.system.ExtrinsicSuccess.is(events[0]));
        break;
      // Fourth event: ethereum.transact:: system.NewAccount, balances.Endowed, (?),
      // ethereum.Executed, system.ExtrinsicSuccess
      case 3:
        t.is(section, "ethereum");
        t.is(method, "transact");
        t.is(events.length, 5);
        t.true(t.context.polkadotApi.events.system.NewAccount.is(events[0]));
        t.true(t.context.polkadotApi.events.balances.Endowed.is(events[1]));
        // TODO: what event was inserted here?
        t.true(t.context.polkadotApi.events.ethereum.Executed.is(events[3]));
        t.true(t.context.polkadotApi.events.system.ExtrinsicSuccess.is(events[4]));
        break;
      default:
        throw new Error(`Unexpected extrinsic`);
    }
  });
});

test("Moonbeam RPC (Balance) - transfer from polkadotjs should appear in ethereum", async (t) => {
  const testAddress = "0x1111111111111111111111111111111111111111";
  const testAddress2 = "0x1111111111111111111111111111111111111112";

  await sendTransfer(t.context.web3, GENESIS_ACCOUNT, testAddress, "0x200");
  await t.context.createAndFinalizeBlock();
  await sendTransfer(t.context.web3, GENESIS_ACCOUNT, testAddress2, 123);
  await t.context.createAndFinalizeBlock();

  t.is(await t.context.web3.eth.getBalance(testAddress2, 2), "123");
});
