import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { start } from "polkadot-launch";
import { typesBundle } from "../moonbeam-types-bundle";
import { createTestPairs } from "@polkadot/keyring/testingPairs";
import type { ParaId } from "@polkadot/types/interfaces";
import { u32 } from "@polkadot/types";

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

async function chain_api(PORT: number): Promise<ApiPromise> {
  const url = `ws://localhost:${PORT}`;
  const provider = new WsProvider(url);
  const api = await ApiPromise.create({
    provider,
    typesBundle: typesBundle as any,
  });
  return api;
}

async function test() {
  await start("config_xcmp.json");
  console.log("done");
  const WS_PORT200 = 36946;
  const WS_PORT201 = 36947;
  const WS_RELAY_PORT = 36944;
  // first Moonbeam Parachain with ID 200
  const moonbeam200 = await chain_api(WS_PORT200);
  // second Moonbeam Parachain with ID 201
  const moonbeam201 = await chain_api(WS_PORT201);
  // relay chain
  const relayApi = await chain_api(WS_RELAY_PORT);
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
  // Open channel using relay sudo as caller
  // const keyring = createTestPairs({ type: "ed25519" }, false);
  // // how do I set these equal to ParaId if ParaId extends u32
  // const sender: u32 = 200;
  // const recipient: u32 = 201;
  // const registerChannel = await relayApi.tx.parasSudoWrapper
  //   .sudoEstablishHrmpChannel(sender, recipient, 8, 1024)
  //   .signAndSend(keyring.alice, ({ events = [], status }) => {
  //     console.log(`Current status is ${status.type}`);

  //     if (status.isFinalized) {
  //       console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

  //       // Loopcod through Vec<EventRecord> to display all events
  //       events.forEach(({ phase, event: { data, method, section } }) => {
  //         console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
  //       });

  //       registerChannel();
  //     }
  //   });
  // construct Transact code to request open channel from 200 -> 201
  const rawOpenCode = relayApi.tx.hrmp.hrmpInitOpenChannel(201, 8, 1024);
  console.log(rawOpenCode.toHex());
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
  console.log("all tests passed");
}
test();
