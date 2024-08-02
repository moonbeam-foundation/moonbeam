import { ApiPromise, Keyring, WsProvider } from "@polkadot/api";
import { start } from "polkadot-launch";
import { typesBundlePre900 } from "../../moonbeam-types-bundle/dist";
import {
  ALITH,
  GERALD,
  FAITH,
  STAKING_AMOUNT,
  ETHAN_PRIVKEY,
  ETHAN,
  DEFAULT_GENESIS_BALANCE,
  ALITH_PRIVKEY,
  MIN_GLMR_NOMINATOR,
  MIN_GLMR_STAKING,
} from "../test-constants";

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
  await start("config_moonbeam_staking.json");
  const WS_PORT = 36946;
  const wsProviderUrl = `ws://localhost:${WS_PORT}`;

  const wsProvider = new WsProvider(wsProviderUrl);
  const polkadotApi = await ApiPromise.create({
    provider: wsProvider,
    typesBundle: typesBundlePre900 as any,
  });

  // subscribe to all new headers (with extended info)
  let lastBlock = Date.now();
  polkadotApi.derive.chain.subscribeNewHeads((header) => {
    console.log(
      `New Block: #${header.number}: ${header.author}, time since last block: ${
        (Date.now() - lastBlock) / 1000
      } sec`
    );
    lastBlock = Date.now();
  });

  // Balance
  const account = await polkadotApi.query.system.account(ETHAN);
  assert(
    account.data.free.toString() === DEFAULT_GENESIS_BALANCE.toString(),
    "wrong balance for Ethan, dif: " + (Number(DEFAULT_GENESIS_BALANCE) - Number(account.data.free))
  );

  // Nominators
  const nominators = await polkadotApi.query.parachainStaking.nominatorState(GERALD);
  assert(nominators.toHuman() === null, "there should be no nominator");

  // Validators
  const validators = await polkadotApi.query.parachainStaking.selectedCandidates();
  assert(validators.toHuman()[0] === GERALD, "Gerald is not a validator");
  assert(validators.toHuman()[1] === FAITH, "Faith is not a validator");

  // Candidates
  const candidates = await polkadotApi.query.parachainStaking.candidatePool();
  assert(candidates.toHuman()[0].owner === GERALD, "Gerald is not a candidates");
  assert(candidates.toHuman()[1].owner === FAITH, "Faith is not a candidates");
  assert(candidates.toHuman()[0].amount === STAKING_AMOUNT, "Gerald has wrong staking amount");
  assert(candidates.toHuman()[1].amount === STAKING_AMOUNT, "Faith has wrong staking amount");

  // Join Candidates
  const keyring = new Keyring({ type: "ethereum" });
  const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
  await new Promise<void>(async (res) => {
    const unsub = await polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING)
      .signAndSend(ethan, ({ events = [], status }) => {
        console.log(`Current status is ${status.type}`);
        if (status.isInBlock) {
          console.log(`Transaction included in Block at blockHash ${status.asInBlock}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        } else if (status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        }
      });
  });
  let candidatesAfter = await polkadotApi.query.parachainStaking.candidatePool();
  assert(
    (candidatesAfter.toHuman() as { owner: string; amount: string }[]).length === 3,
    "new candidate should have been added"
  );
  assert(
    (candidatesAfter.toHuman() as { owner: string; amount: string }[])[2].owner === ETHAN,
    "new candidate ethan should have been added"
  );
  assert(
    (candidatesAfter.toHuman() as { owner: string; amount: string }[])[2].amount === "1.0000 kUnit",
    "new candidate ethan should have been added (wrong amount)"
  );

  // Candidate bond more
  await new Promise<void>(async (res) => {
    const unsub = await polkadotApi.tx.parachainStaking
      .candidateBondMore(MIN_GLMR_STAKING)
      .signAndSend(ethan, ({ events = [], status }) => {
        console.log(`Current status is ${status.type}`);
        if (status.isInBlock) {
          console.log(`Transaction included in Block at blockHash ${status.asInBlock}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        } else if (status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        }
      });
  });
  candidatesAfter = await polkadotApi.query.parachainStaking.candidatePool();
  assert(
    (candidatesAfter.toHuman() as { owner: string; amount: string }[])[2].amount === "2.0000 kUnit",
    "bond should have increased"
  );

  // Candidate bond less
  await new Promise<void>(async (res) => {
    const unsub = await polkadotApi.tx.parachainStaking
      .candidateBondLess(MIN_GLMR_STAKING)
      .signAndSend(ethan, ({ events = [], status }) => {
        console.log(`Current status is ${status.type}`);
        if (status.isInBlock) {
          console.log(`Transaction included in Block at blockHash ${status.asInBlock}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        } else if (status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        }
      });
  });
  candidatesAfter = await polkadotApi.query.parachainStaking.candidatePool();
  assert(
    (candidatesAfter.toHuman() as { owner: string; amount: string }[])[2].amount === "1.0000 kUnit",
    "bond should have decreased"
  );

  // Join Nominators
  const keyringAlith = new Keyring({ type: "ethereum" });
  const alith = await keyringAlith.addFromUri(ALITH_PRIVKEY, null, "ethereum");
  await new Promise<void>(async (res) => {
    const unsub = await polkadotApi.tx.parachainStaking
      .nominate(GERALD, MIN_GLMR_NOMINATOR)
      .signAndSend(alith, ({ events = [], status }) => {
        console.log(`Current status is ${status.type}`);
        if (status.isInBlock) {
          console.log(`Transaction included in Block at blockHash ${status.asInBlock}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        } else if (status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

          // Loop through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        }
      });
  });
  const nominatorsAfter = await polkadotApi.query.parachainStaking.nominatorState(ALITH);
  assert(
    (
      nominatorsAfter.toHuman() as {
        nominations: { owner: string; amount: string }[];
      }
    ).nominations[0].owner === GERALD,
    "nomination didn't go through"
  );

  // Revoke Delegation
  await new Promise<void>(async (res) => {
    const unsub = await polkadotApi.tx.parachainStaking
      .revokeDelegation(GERALD) //TODO: when converting to test add .leaveNominators()
      // that should produce the same behavior
      .signAndSend(alith, ({ events = [], status }) => {
        console.log(`Current status is ${status.type}`);
        if (status.isInBlock) {
          console.log(`Transaction included in Block at blockHash ${status.asInBlock}`);

          // Loopcod through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        } else if (status.isFinalized) {
          console.log(`Transaction finalized at blockHash ${status.asFinalized}`);

          // Loop through Vec<EventRecord> to display all events
          events.forEach(({ phase, event: { data, method, section } }) => {
            console.log(`\t' ${phase}: ${section}.${method}:: ${data}`);
          });

          unsub();
          res();
        }
      });
  });
  const nominatorsAfterRevocation = await polkadotApi.query.parachainStaking.nominatorState(ALITH);
  assert(nominatorsAfterRevocation.toHuman() === null, "there should be no nominator");

  console.log("SUCCESS");
}
test();

// TODO: leave_candidates
// TODO: ethan (added candidate) doesn't produce blocks => need to move blockPerRound to storage
