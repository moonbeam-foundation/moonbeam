import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { start } from "polkadot-launch";
import { typesBundle } from "../moonbeam-types-bundle";
import { createTestPairs } from "@polkadot/keyring/testingPairs";
import type { ParaId } from "@polkadot/types/interfaces";
import { u32 } from "@polkadot/types";
import { gerald } from "./init-web3";

// constants
const GERALD = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const GERALD_PRIVKEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
const ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
const STAKING_AMOUNT = "1.0000 kUnit";
const GLMR = 1_000_000_000_000_000_000n;
const MIN_GLMR_STAKING = 1000n * GLMR;
const MIN_GLMR_NOMINATOR = 5n * GLMR;
const DEFAULT_GENESIS_BALANCE = 2n ** 80n;
const DEFAULT_GENESIS_STAKING = 1_000n * GLMR;
const GENESIS_ACCOUNT_BALANCE = DEFAULT_GENESIS_BALANCE - DEFAULT_GENESIS_STAKING;

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

async function wait(duration: number) {
  console.log(`Waiting ${duration / 1000} seconds`);
  return new Promise((res) => {
    setTimeout(res, duration);
  });
}

async function parachain_api(PORT: number): Promise<ApiPromise> {
  const url = `ws://localhost:${PORT}`;
  const provider = new WsProvider(url);
  const api = await ApiPromise.create({
    provider,
    typesBundle: typesBundle as any,
  });
  return api;
}

async function relaychain_api(PORT: number): Promise<ApiPromise> {
  const url = `ws://localhost:${PORT}`;
  const provider = new WsProvider(url);
  const api = await ApiPromise.create({
    provider,
    types: {
      Address: "MultiAddress",
      LookupSource: "MultiAddress",
    },
  });
  return api;
}

async function test() {
  await start("config_xcmp.json");
  console.log("moonbeam launch launched");
  const WS_PORT200 = 36946;
  const WS_PORT201 = 36947;
  const WS_RELAY_PORT = 36944;
  // first Moonbeam Parachain with ID 200
  const moonbeam200 = await parachain_api(WS_PORT200);
  // second Moonbeam Parachain with ID 201
  const moonbeam201 = await parachain_api(WS_PORT201);
  // relay chain
  const relayApi = await relaychain_api(WS_RELAY_PORT);
  // sanity checks that genesis state for all chains meet expectations
  const gerald200 = await moonbeam200.query.system.account(GERALD);
  assert(
    gerald200.data.free.toString() === GENESIS_ACCOUNT_BALANCE.toString(),
    "wrong balance for Gerald, dif: " +
      (Number(GENESIS_ACCOUNT_BALANCE) - Number(gerald200.data.free))
  );
  const gerald201 = await moonbeam201.query.system.account(GERALD);
  assert(
    gerald201.data.free.toString() === GENESIS_ACCOUNT_BALANCE.toString(),
    "wrong balance for Gerald, dif: " +
      (Number(GENESIS_ACCOUNT_BALANCE) - Number(gerald201.data.free))
  );
  const relayAlice = await relayApi.query.system.account(ALICE);
  assert(
    "1000000000000000000" === relayAlice.data.free.toString(),
    "wrong balance for relayAlice, expected: 1000000000000000000, returned: " +
      Number(relayAlice.data.free)
  );
  console.log("Sanity Checks Passed for Relay Chain and Both Parachains");
  // Open channel using relay sudo as caller
  const keyring = new Keyring({ type: "sr25519" });
  const alice = keyring.addFromUri("//Alice");
  const sender: number = 200;
  const recipient: number = 201;
  const unsub = await relayApi.tx.sudo
    .sudo(relayApi.tx.parasSudoWrapper.sudoEstablishHrmpChannel(sender, recipient, 8, 1024))
    .signAndSend(alice, {}, (result) => {
      console.log(`Current status is ${result.status}`);
      if (result.status.isInBlock) {
        console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
      } else if (result.status.isFinalized) {
        console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
        unsub();
      }
    });
  // (1) TODO: check that the channel is actually open by querying relay chain storage for `hrmp` pallet
  // const channelID = HrmpChannelId { sender, recipient };
  // const expectedChannel = await relayApi.query.hrmp.hrmpChannels(channelID);
  // assert(
  //   expectedChannel.isSome(),
  //   "Channel does not exist but we expected it to exist"
  // );
  // (2) TODO: check that channel deposits are reserved from sender and recipient
  // HOW LONG TO WAIT UNTIL QUEUED DOWNWARD MESSAGES ARE RECEIVED BY PARARCHAIN
  await wait(50000);
  // (3) TODO: check that the downward message Xcm::HrmpNewChannelOpenRequest { sender, max_msg_size, max_capacity }
  //  was sent to the recipient parachain
  // const recipientChannels = moonbeam201.query.channels.recipientChannels();
  // assert(
  //   senderChannels[0] === recipient,
  //   "Sender channel with recipient ID not yet opened on sender chain"
  // );
  // assert(
  //   recipientChannels[0] === sender,
  //   "Recipient channel with sender ID not yet opened on recipient chain"
  // );
  // (4) TODO: check that the downward message Xcm::HrmpChannelAccepted { recipient } was sent to the sender parachain
  // const senderChannels = moonbeam200.query.channels.senderChannels();
  // assert(
  //   senderChannels[0] === recipient,
  //   "Sender channel with recipient ID not yet opened on sender chain"
  // );
  // (5) Transfer from Sender to Recipient Parachain
  // transfer_native_to_account_key_20_parachain
  // const senderKeyring = new Keyring({ type: "ethereum" });
  // const gerald = await senderKeyring.addFromUri(GERALD_PRIVKEY, null, "ethereum");
  // const unsub2 = await moonbeam200.tx.xtransfer
  //   .transferNative(recipient, GERALD, 100000)
  //   .signAndSend(gerald, ({ events = [], status }) => {
  //     console.log(`Current status is ${status.type}`);

  //     if (status.isFinalized) {
  //       console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

  //       // Loopcod through Vec<EventRecord> to display all events
  //       events.forEach(({ phase, event: { data, method, section } }) => {
  //         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
  //       });

  //       unsub2();
  //     }
  //   });
  // check to see if message is received on the recipient chain
  // check to see if account balance changes on sender chain
  // check to see if account balance changes on recipient chain
  // (6) Test transfer in the opposite direction, register a new channel first and etc
  // const sender2: number = recipient;
  // const recipient2: number = sender;
  // const unsub3 = await relayApi.tx.sudo
  //   .sudo(relayApi.tx.parasSudoWrapper.sudoEstablishHrmpChannel(sender2, recipient2, 8, 1024))
  //   .signAndSend(alice, {}, (result) => {
  //     console.log(`Current status is ${result.status}`);
  //     if (result.status.isInBlock) {
  //       console.log(`Transaction included at blockHash ${result.status.asInBlock}`);
  //     } else if (result.status.isFinalized) {
  //       console.log(`Transaction finalized at blockHash ${result.status.asFinalized}`);
  //       unsub3();
  //     }
  //   });
  console.log("all tests passed");
}
test();

// construct Transact code to request open channel from 200 -> 201
// const rawOpenCode = relayApi.tx.hrmp.hrmpInitOpenChannel(201, 8, 1024);
// console.log(rawOpenCode.toHex());
// const rawParaCallTest = moonbeam200.tx.parachainStaking.leaveNominators();
// console.log(rawParaCallTest.toHex());
// remove prefix 31c04
// const openCode = "0x1600c90000000800000000040000";
// // Send message from 200 to relay to request open channel from 200 -> 201
// const keyring = new Keyring({ type: "ethereum" });
// const gerald = await keyring.addFromUri(GERALD_PRIVKEY, null, "ethereum");
// const unsub = await moonbeam200.tx.xtransfer
//   .openChannel(201, openCode)
//   .signAndSend(gerald, ({ events = [], status }) => {
//     console.log(`Current status is ${status.type}`);

//     if (status.isFinalized) {
//       console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

//       // Loopcod through Vec<EventRecord> to display all events
//       events.forEach(({ phase, event: { data, method, section } }) => {
//         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
//       });

//       unsub();
//     }
//   });
// Send message from 201 to relay to accept channel request from 200 -> 201
// Transfer Moonbeam from 200 to 201
