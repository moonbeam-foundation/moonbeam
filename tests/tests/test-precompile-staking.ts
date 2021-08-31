import { expect } from "chai";
import Keyring from "@polkadot/keyring";
import {
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
  COLLATOR_ACCOUNT,
  ETHAN_PRIVKEY,
  MIN_GLMR_STAKING,
  ETHAN,
  ALITH,
  MIN_GLMR_NOMINATOR,
  GENESIS_ACCOUNT,
  ALITH_PRIV_KEY,
} from "../util/constants";
import { blake2AsHex, randomAsHex } from "@polkadot/util-crypto";
import { describeDevMoonbeam, DevTestContext } from "../util/setup-dev-tests";
import { numberToHex, stringToHex } from "@polkadot/util";
import Web3 from "web3";
import { customWeb3Request } from "../util/providers";
import { createTransaction } from "../util/transactions";

const ADDRESS_STAKING = "0x0000000000000000000000000000000000000800";

const SELECTORS = {
  candidate_bond_less: "289b6ba7",
  candidate_bond_more: "c57bd3a8",
  go_offline: "767e0450",
  go_online: "d2f73ceb",
  is_candidate: "8545c833",
  is_selected_candidate: "8f6d27c7",
  is_nominator: "8e5080e7",
  join_candidates: "0a1bff60",
  leave_candidates: "72b02a31",
  leave_nominators: "b71d2153",
  min_nomination: "c9f593b2",
  nominate: "49df6eb3",
  nominator_bond_less: "f6a52569",
  nominator_bond_more: "971d44c8",
  revoke_nomination: "4b65c34b",
  points: "9799b4e7",
  //
  candidate_count: "4b1c4c29",
};
const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

async function isSelectedCandidate(context: DevTestContext, address: string) {
  const addressData = address.slice(2).padStart(64, "0");

  return await customWeb3Request(context.web3, "eth_call", [
    {
      from: GENESIS_ACCOUNT,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_STAKING,
      data: `0x${SELECTORS.is_selected_candidate}${addressData}`,
    },
  ]);
}

async function candidateCount(context: DevTestContext) {
  console.log("candidate_count()");
  console.log(Web3.utils.sha3("candidate_count()"));

  return await customWeb3Request(context.web3, "eth_call", [
    {
      from: GENESIS_ACCOUNT,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_STAKING,
      data: `0x${SELECTORS.candidate_count}`,
    },
  ]);
}

async function joinCandidates(
  context: DevTestContext,
  amount: number,
  candidateCount: number,
  privateKey: string,
  from: string
) {
  const amountData = numberToHex(amount).slice(2).padStart(64, "0");
  const candidateCountData = numberToHex(candidateCount).slice(2).padStart(64, "0");

  let data = `0x${SELECTORS.join_candidates}${amountData}${candidateCountData}`;

  const tx = await createTransaction(context.web3, {
    from,
    privateKey,
    value: "0x0",
    gas: "0x200000",
    gasPrice: GAS_PRICE,
    to: ADDRESS_STAKING,
    data,
  });

  const block = await context.createBlock({
    transactions: [tx],
  });
  const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
  expect(receipt.status).to.equal(true);
  return block;
}

async function candidateBondMore(context: DevTestContext, amount: number, privateKey, from) {
  const amountData = numberToHex(amount).slice(2).padStart(64, "0");

  let data = `0x${SELECTORS.candidate_bond_more}${amountData}`;

  const tx = await createTransaction(context.web3, {
    from,
    privateKey,
    value: "0x0",
    gas: "0x200000",
    gasPrice: GAS_PRICE,
    to: ADDRESS_STAKING,
    data,
  });

  const block = await context.createBlock({
    transactions: [tx],
  });
  return block;
}

describeDevMoonbeam("Staking - Genesis", (context) => {
  //   it("should match collator reserved bond reserved", async function () {
  //     console.log("ok", blake2AsHex("balanceOf"));
  //     console.log("ok", stringToHex("balanceOf"));
  //     console.log(Web3.utils.sha3("balanceOf"));
  //     console.log(Web3.utils.sha3("balanceOf(address)"));
  //     const account = await context.polkadotApi.query.system.account(COLLATOR_ACCOUNT);
  //     const expectedReserved = DEFAULT_GENESIS_STAKING + DEFAULT_GENESIS_MAPPING;
  //     expect(account.data.reserved.toString()).to.equal(expectedReserved.toString());
  //   });

  it("should include collator from the specs", async function () {
    // const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    // expect((collators[0] as Buffer).toString("hex").toLowerCase()).equal(COLLATOR_ACCOUNT);
    // console.log("oh");
    // console.log(await isSelectedCandidate(context, COLLATOR_ACCOUNT));
    expect(Number((await isSelectedCandidate(context, COLLATOR_ACCOUNT)).result)).to.equal(1);
  });
  it.only("should have one collator", async function () {
    // const collators = await context.polkadotApi.query.parachainStaking.selectedCandidates();
    // expect((collators[0] as Buffer).toString("hex").toLowerCase()).equal(COLLATOR_ACCOUNT);
    // console.log("oh");
    // console.log(await isSelectedCandidate(context, COLLATOR_ACCOUNT));
    expect(Number((await candidateCount(context)).result)).to.equal(1);
  });
});

async function getBalance(context, address) {
  return (await context.polkadotApi.query.system.account(address)).data.free.toHuman();
}

describeDevMoonbeam("Staking - Join Candidates", (context) => {
  it.only("should succesfully call joinCandidates on ETHAN", async function () {
    // const keyring = new Keyring({ type: "ethereum" });
    // const ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    // await context.polkadotApi.tx.parachainStaking
    //   .joinCandidates(MIN_GLMR_STAKING, 1)
    //   .signAndSend(ethan);
    // await context.createBlock();

    console.log(" MIN_GLMR_STAKING.toString()", MIN_GLMR_STAKING.toString());
    console.log("balance ethan", await getBalance(context, ETHAN));
    console.log("numberToHex(Number(MIN_GLMR_STAKING),64)",numberToHex(Number(MIN_GLMR_STAKING) ))
    await joinCandidates(context, Number(MIN_GLMR_STAKING), 1, ETHAN_PRIVKEY, ETHAN);

    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    console.log(candidatesAfter.toHuman());
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[]).length === 2
    ).to.equal(true, "new candidate should have been added");
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[1].owner === ETHAN
    ).to.equal(true, "new candidate ethan should have been added");
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[1].amount ===
        "1.0000 kUNIT"
    ).to.equal(true, "new candidate ethan should have been added (wrong amount)");
  });
});

describeDevMoonbeam("Staking - Candidate bond more", (context) => {
  let ethan;
  before("should succesfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();
  });
  it("should succesfully call candidateBondMore on ETHAN", async function () {
    await context.polkadotApi.tx.parachainStaking
      .candidateBondMore(MIN_GLMR_STAKING)
      .signAndSend(ethan);
    await context.createBlock();
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[1].amount ===
        "2.0000 kUNIT"
    ).to.equal(true, "bond should have increased");
  });
  it.only("should succesfully call candidateBondMore on ALITH", async function () {
    await candidateBondMore(context, Number(MIN_GLMR_STAKING), ALITH_PRIV_KEY, ALITH);
    await context.createBlock();
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    console.log(candidatesAfter.toHuman());
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[0].amount ===
        "2.0000 kUNIT"
    ).to.equal(true, "bond should have increased");
  });
});

describeDevMoonbeam("Staking - Candidate bond less", (context) => {
  let ethan;
  before("should succesfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .joinCandidates(MIN_GLMR_STAKING, 1)
      .signAndSend(ethan);
    await context.createBlock();
  });
  it("should succesfully call candidateBondLess on ETHAN", async function () {
    await context.polkadotApi.tx.parachainStaking
      .candidateBondLess(MIN_GLMR_STAKING)
      .signAndSend(ethan);
    await context.createBlock();
    let candidatesAfter = await context.polkadotApi.query.parachainStaking.candidatePool();
    expect(
      (candidatesAfter.toHuman() as { owner: string; amount: string }[])[1].amount ===
        "1.0000 kUNIT"
    ).to.equal(true, "bond should have decreased");
  });
});

describeDevMoonbeam("Staking - Join Nominators", (context) => {
  let ethan;
  beforeEach("should succesfully call joinCandidates on ETHAN", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
    await context.polkadotApi.tx.parachainStaking
      .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
      .signAndSend(ethan);
    await context.createBlock();
  });
  it("should succesfully call nominate on ALITH", async function () {
    const nominatorsAfter = await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN);
    expect(
      (
        nominatorsAfter.toHuman() as {
          nominations: { owner: string; amount: string }[];
        }
      ).nominations[0].owner === ALITH
    ).to.equal(true, "nomination didnt go through");
    expect(Object.keys(nominatorsAfter.toHuman()["status"])[0]).equal("Active");
  });
  it("should succesfully revoke nomination on ALITH", async function () {
    await context.polkadotApi.tx.parachainStaking.revokeNomination(ALITH).signAndSend(ethan);
    await context.createBlock();

    const nominatorsAfter = await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN);
    expect(nominatorsAfter.toHuman()["status"].Leaving).equal("3");
  });
});

// // TODO: bring back when we figure out how to get `NominatorState2.revocations`
// describeDevMoonbeam("Staking - Revoke Nomination", (context) => {
//   let ethan;
//   before("should succesfully call nominate on ALITH", async function () {
//     //nominate
//     const keyring = new Keyring({ type: "ethereum" });
//     ethan = await keyring.addFromUri(ETHAN_PRIVKEY, null, "ethereum");
//     await context.polkadotApi.tx.parachainStaking
//       .nominate(ALITH, MIN_GLMR_NOMINATOR, 0, 0)
//       .signAndSend(ethan);
//     await context.createBlock();
//   });
//   it("should succesfully revoke nomination for ALITH", async function () {
//     await context.polkadotApi.tx.parachainStaking.revokeNomination(ALITH).signAndSend(ethan);
//     await context.createBlock();
//     const nominatorsAfterRevocation =
//       await context.polkadotApi.query.parachainStaking.nominatorState2(ETHAN);
//     expect(
//       (nominatorsAfterRevocation.revocations[0] === ALITH).to.equal(
//         true,
//         "revocation didnt go through"
//       )
//     );
//   });
// });
