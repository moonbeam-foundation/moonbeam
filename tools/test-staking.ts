import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { start } from "polkadot-launch";
import { typesBundle } from "../moonbeam-types-bundle";
import {
  GERALD,
  FAITH,
  STAKING_AMOUNT,
  ETHAN_PRIVKEY,
  ETHAN,
  GENESIS_ACCOUNT_BALANCE,
  DEFAULT_GENESIS_BALANCE,
  ALITH_PRIVKEY,
} from "./test-constants";

function assert(condition: boolean, msg: string) {
  if (!condition) throw new Error(msg);
}

async function wait(duration: number) {
  console.log(`Waiting ${duration / 1000} seconds`);
  return new Promise((res) => {
    setTimeout(res, duration);
  });
}

async function test() {
  await start("config_moonbeam.json");
  console.log("done");
  const WS_PORT = 36946;
  const wsProviderUrl = `ws://localhost:${WS_PORT}`;

  const wsProvider = new WsProvider(wsProviderUrl);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundle as any,
  });

  // Balance
  const account = await polkadotApi.query.system.account(ETHAN);
  assert(
    account.data.free.toString() === DEFAULT_GENESIS_BALANCE.toString(),
    "wrong balance for Ethan"
  );

  // Nominators
  const nominators = await polkadotApi.query.stake.nominators(GERALD);
  console.log("he", nominators.toHuman());
  assert(nominators.toHuman() === null, "there should be no nominator");

  // Validators
  const validators = await polkadotApi.query.stake.validators();
  assert(validators.toHuman()[0].toLowerCase() === GERALD, "Gerald is not a validator");
  assert(validators.toHuman()[1].toLowerCase() === FAITH.toLowerCase(), "Faith is not a validator");

  // Candidates
  const candidates = await polkadotApi.query.stake.candidatePool();
  console.log("candodates", candidates.toHuman());
  assert(candidates.toHuman()[0].owner.toLowerCase() === GERALD, "Gerald is not a candidates");
  assert(
    candidates.toHuman()[1].owner.toLowerCase() === FAITH.toLowerCase(),
    "Faith is not a candidates"
  );
  assert(candidates.toHuman()[0].amount === STAKING_AMOUNT, "Gerald has wrong staking amount");
  assert(candidates.toHuman()[1].amount === STAKING_AMOUNT, "Faith has wrong staking amount");

  // Join Candidates
  const keyring = new Keyring({ type: "ethereum" });
  const testAccount = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
  await polkadotApi.tx.stake.joinCandidates(0, "100000000000000000000000").signAndSend(testAccount);
  await wait(10000);
  const candidatesAfter = await polkadotApi.query.stake.candidatePool();
  console.log("candidatesAfter", candidatesAfter.toHuman());
  const ethan = await polkadotApi.query.system.account(ETHAN);
  console.log(ethan.data.free.toString());
  console.log(DEFAULT_GENESIS_BALANCE.toString());

  // Join Nominators
  const keyringAlith = new Keyring({ type: "ethereum" });
  const alith = await keyringAlith.addFromUri(ALITH_PRIVKEY, null, "ethereum");
  await polkadotApi.tx.stake.joinNominators(GERALD, 10).signAndSend(alith);
  await wait(10000);
  const nominatorsAfter = await polkadotApi.query.stake.nominators(GERALD);
  console.log("nominatorsAfter", nominatorsAfter.toHuman());

  console.log("SUCCESS");
}
test();
