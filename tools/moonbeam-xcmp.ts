import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { start } from "polkadot-launch";
import { typesBundle } from "../moonbeam-types-bundle";

// constants
const GERALD = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
const FAITH = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";
const ETHAN_PRIVKEY = "0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4";
const ETHAN = "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB";
const ALITH_PRIVKEY = "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
const ALITH = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
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
  const ALICE = "5GrwvaEF5zXb26Fz9rcQpDWS57CtERHpNehXCPcNoHGKutQY";
  const relayAlice = await relayApi.query.system.account(ALICE);
  assert(
    "1000000000000000000" === relayAlice.data.free.toString(),
    "wrong balance for relayAlice, expected: 1000000000000000000, returned: " +
      Number(relayAlice.data.free)
  );
  console.log("all tests passed");
}
test();
