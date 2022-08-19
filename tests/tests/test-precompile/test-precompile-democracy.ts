import "@moonbeam-network/api-augment";

import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { u32 } from "@polkadot/types-codec";
import { BN, numberToHex } from "@polkadot/util";
import { blake2AsHex } from "@polkadot/util-crypto";
import { expect } from "chai";
import { ethers } from "ethers";
import { Interface } from "ethers/lib/utils";

import { alith, ALITH_PRIVATE_KEY, baltathar, BALTATHAR_PRIVATE_KEY } from "../../util/accounts";
import {
  PRECOMPILE_DEMOCRACY_ADDRESS,
  PROPOSAL_AMOUNT,
  VOTE_AMOUNT,
  ZERO_ADDRESS,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  sendPrecompileTx,
} from "../../util/transactions";

// Function selector reference
// {
// "0185921e": "delegate(address,uint256,uint256)",
// "a30305e9": "deposit_of(uint256)",
// "b1fd383f": "finished_referendum_info(uint256)",
// "0388f282": "lowest_unbaked()",
// "8b93d11a": "ongoing_referendum_info(uint256)",
// "7824e7d1": "propose(bytes32,uint256)",
// "56fdf547": "public_prop_count()",
// "2042f50b": "remove_vote(uint256)",
// "c7a76601": "second(uint256,uint256)",
// "3f3c21cc": "standard_vote(uint256,bool,uint256,uint256)",
// "cb37b8ea": "un_delegate()",
// "2f6c493c": "unlock(address)"
// }

const SELECTORS = {
  propose: "7824e7d1",
  second: "c7a76601",
  standardVote: "6cd18b0d",
  notePreimage: "cb00f603",
};

const DEMOCRACY_INTERFACE = new ethers.utils.Interface(getCompiled("Democracy").contract.abi);

export const deployContract = async (context: DevTestContext, contractName: string) => {
  const { rawTx } = await createContract(context, contractName);
  await context.createBlock(rawTx);
};

export const notePreimagePrecompile = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes
>(
  context: DevTestContext,
  iFace: Interface,
  proposal: Call
): Promise<`0x${string}`> => {
  const encodedProposal = proposal.method.toHex();

  const data = iFace.encodeFunctionData(
    // action
    "notePreimage",
    [encodedProposal]
  );

  const tx = await createTransaction(context, {
    ...ALITH_TRANSACTION_TEMPLATE,
    to: PRECOMPILE_DEMOCRACY_ADDRESS,
    data,
  });

  await context.createBlock(tx);
  // return encodedHash
  return blake2AsHex(encodedProposal);
};

describeDevMoonbeam("Democracy - genesis and preimage", (context) => {
  before("Setup genesis account for substrate", async () => {
    await deployContract(context, "Democracy");
  });
  it("should check initial state - no referendum", async function () {
    // referendumCount
    const referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toNumber()).to.equal(0);
  });
  it("should check initial state - 0x0 ParachainBondAccount", async function () {
    // referendumCount
    const parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();
    expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
  });
  it("notePreimage", async function () {
    // notePreimage
    const encodedHash = await notePreimagePrecompile(
      context,
      DEMOCRACY_INTERFACE,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
    );

    const preimageStatus = await context.polkadotApi.query.democracy.preimages(encodedHash);
    expect(
      preimageStatus.unwrap().isAvailable && preimageStatus.unwrap().asAvailable.provider.toString()
    ).to.equal(alith.address);
    expect(
      preimageStatus.unwrap().isAvailable && preimageStatus.unwrap().asAvailable.deposit.toString()
    ).to.equal("2200000000000000");
  });
});

describeDevMoonbeam("Democracy - propose", (context) => {
  let encodedHash: `0x${string}`;

  before("Setup genesis account for substrate", async () => {
    await deployContract(context, "Democracy");

    // encodedHash
    encodedHash = await notePreimagePrecompile(
      context,
      DEMOCRACY_INTERFACE,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
    );
  });
  it("propose", async function () {
    // propose
    await sendPrecompileTx(
      context,
      PRECOMPILE_DEMOCRACY_ADDRESS,
      SELECTORS,
      alith.address,
      ALITH_PRIVATE_KEY,
      "propose",
      [encodedHash, numberToHex(Number(PROPOSAL_AMOUNT))]
    );

    // referendumCount
    const referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toNumber()).to.equal(0);

    // publicPropCount
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toNumber()).to.equal(1);

    // publicProps
    const publicProps = await context.polkadotApi.query.democracy.publicProps();
    // encodedHash
    expect(publicProps[0][1].toString()).to.equal(encodedHash);
    // prop author
    expect(publicProps[0][2].toString()).to.equal(alith.address);
    // depositOf
    const depositOf = await context.polkadotApi.query.democracy.depositOf(0);
    expect(depositOf.unwrap()[1].toBigInt()).to.equal(1_000_000_000_000_000_000_000n);
  });
});

describeDevMoonbeam("Democracy - second proposal", (context) => {
  let encodedHash: `0x${string}`;
  let launchPeriod: u32;

  before("Setup genesis account for substrate", async () => {
    await deployContract(context, "Democracy");

    //launchPeriod
    launchPeriod = context.polkadotApi.consts.democracy.launchPeriod;

    // notePreimage
    encodedHash = await notePreimagePrecompile(
      context,
      DEMOCRACY_INTERFACE,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
    );

    // propose
    await sendPrecompileTx(
      context,
      PRECOMPILE_DEMOCRACY_ADDRESS,
      SELECTORS,
      alith.address,
      ALITH_PRIVATE_KEY,
      "propose",
      [encodedHash, numberToHex(Number(PROPOSAL_AMOUNT))]
    );

    // second
    await sendPrecompileTx(
      context,
      PRECOMPILE_DEMOCRACY_ADDRESS,
      SELECTORS,
      baltathar.address,
      BALTATHAR_PRIVATE_KEY,
      "second",
      [numberToHex(0), numberToHex(1000)]
    );
  });
  // TODO: test getters
  it("second proposal", async function () {
    // publicProps
    const publicProps = await context.polkadotApi.query.democracy.publicProps();
    // encodedHash
    expect(publicProps[0][1].toString()).to.equal(encodedHash);
    // prop author
    expect(publicProps[0][2].toString()).to.equal(alith.address);

    // depositOf
    const depositOf = await context.polkadotApi.query.democracy.depositOf(0);
    expect(depositOf.unwrap()[1].toBigInt()).to.equal(1_000_000_000_000_000_000_000n);
    expect(depositOf.unwrap()[0][1].toString()).to.equal(baltathar.address);
  });

  it("check referendum is up", async function () {
    this.timeout(1000000);
    // let Launchperiod elapse to turn the proposal into a referendum
    // launchPeriod minus the 3 blocks that already elapsed
    for (let i = 0; i < Number(launchPeriod) - 3; i++) {
      await context.createBlock();
    }
    // referendumCount
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toNumber()).to.equal(1);

    // publicPropCount
    const publicPropCount = await context.polkadotApi.query.democracy.publicPropCount();
    expect(publicPropCount.toNumber()).to.equal(1);

    // referendumInfoOf
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect(referendumInfoOf.unwrap().asOngoing.proposalHash.toString()).to.equal(encodedHash);
  });
});

describeDevMoonbeam("Democracy - vote on referendum", (context) => {
  let encodedHash: `0x${string}`;
  let enactmentPeriod: u32;

  before("Setup genesis account for substrate", async () => {
    await deployContract(context, "Democracy");

    // enactmentPeriod
    enactmentPeriod = await context.polkadotApi.consts.democracy.enactmentPeriod;

    // encodedHash
    encodedHash = await notePreimagePrecompile(
      context,
      DEMOCRACY_INTERFACE,
      context.polkadotApi.tx.parachainStaking.setParachainBondAccount(alith.address)
    );

    // propose
    await sendPrecompileTx(
      context,
      PRECOMPILE_DEMOCRACY_ADDRESS,
      SELECTORS,
      alith.address,
      ALITH_PRIVATE_KEY,
      "propose",
      [encodedHash, numberToHex(Number(PROPOSAL_AMOUNT))]
    );

    // second
    await sendPrecompileTx(
      context,
      PRECOMPILE_DEMOCRACY_ADDRESS,
      SELECTORS,
      baltathar.address,
      BALTATHAR_PRIVATE_KEY,
      "second",
      [numberToHex(0), numberToHex(1000)]
    );
  });

  it("vote", async function () {
    this.timeout(2000000);
    // let Launchperiod elapse to turn the proposal into a referendum
    // launchPeriod minus the 3 blocks that already elapsed
    for (let i = 0; i < 7200 - 3; i++) {
      await context.createBlock();
    }
    // vote
    await sendPrecompileTx(
      context,
      PRECOMPILE_DEMOCRACY_ADDRESS,
      SELECTORS,
      alith.address,
      ALITH_PRIVATE_KEY,
      "standardVote",
      [numberToHex(0), "0x01", numberToHex(Number(VOTE_AMOUNT)), numberToHex(1)]
    );
    // referendumInfoOf
    const referendumInfoOf = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect(referendumInfoOf.unwrap().asOngoing.proposalHash.toHex()).to.equal(encodedHash);
    expect(referendumInfoOf.unwrap().asOngoing.delay.toNumber()).to.equal(
      enactmentPeriod.toNumber()
    );
    expect(referendumInfoOf.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
      10_000_000_000_000_000_000n
    );
    expect(referendumInfoOf.unwrap().asOngoing.tally.turnout.toBigInt()).to.equal(
      10_000_000_000_000_000_000n
    );

    const referendumHex = referendumInfoOf.toHex();

    // Instead of waiting votePeriod + enactmentPeriod (which is very long) we hack
    // the referendum to be shorter
    const blockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number;

    // new end block to
    const newEndBlock = context.polkadotApi.registry.createType(
      "u32",
      blockNumber.toBn().add(new BN(2))
    );

    // Set 0 block delay
    const delay = context.polkadotApi.registry.createType(
      "u32",
      referendumInfoOf.unwrap().asOngoing.delay.sub(enactmentPeriod)
    );

    // taking same referendum with different end & delay
    const modReferendum = `0x00${newEndBlock.toHex(true).slice(2)}${referendumHex.slice(
      12,
      78
    )}${delay.toHex(true).slice(2)}${referendumHex.slice(86)}`;

    // Changing storage for the referendum using sudo
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [context.polkadotApi.query.democracy.referendumInfoOf.key(0).toString(), modReferendum],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    // Waiting extra blocks for the vote to finish
    for (let i = 0; i < 2; i++) {
      await context.createBlock();
    }

    let parachainBondInfo = await context.polkadotApi.query.parachainStaking.parachainBondInfo();

    const referendumDone = await context.polkadotApi.query.democracy.referendumInfoOf(0);
    expect(referendumDone.unwrap().isFinished).to.be.true;
    expect(referendumDone.unwrap().asFinished.approved.isTrue).to.be.true;
    expect(parachainBondInfo.account.toString()).to.equal(alith.address);
  });
});
