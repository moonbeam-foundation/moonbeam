import "@moonbeam-network/api-augment";
import Debug from "debug";
import Contract from "web3-eth-contract";
import { expect } from "chai";
import { ethers } from "ethers";
import { BALTATHAR_PRIVATE_KEY, alith, baltathar } from "../../util/accounts";
import { PRECOMPILE_CONVICTION_VOTING_ADDRESS } from "../../util/constants";

import { getCompiled } from "../../util/contracts";

import { expectOk, expectSubstrateEvent } from "../../util/expect";

import { DevTestContext, describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContractExecution } from "../../util/transactions";
import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";
import { jumpBlocks } from "../../util/block";
const debug = Debug("test:precompile-conviction-voting");

const CONVICTION_VOTING_CONTRACT = getCompiled("precompiles/conviction-voting/ConvictionVoting");
const CONVICTION_VOTING_INTERFACE = new ethers.utils.Interface(
  CONVICTION_VOTING_CONTRACT.contract.abi
);

const convictionVotingContract = new Contract(
  CONVICTION_VOTING_CONTRACT.contract.abi,
  PRECOMPILE_CONVICTION_VOTING_ADDRESS
);

async function createProposal(context: DevTestContext, track = "root") {
  let nonce = (await context.polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();
  const call = context.polkadotApi.tx.identity.setIdentity({ display: { raw: "Me" } });
  const block = await context.createBlock([
    context.polkadotApi.tx.preimage.notePreimage(call.toHex()).signAsync(alith, { nonce: nonce++ }),
    context.polkadotApi.tx.referenda
      .submit(
        track == "root" ? { system: "root" } : { Origins: track },
        { Lookup: { Hash: call.hash.toHex(), len: call.length } },
        { After: 1 }
      )
      .signAsync(alith, { nonce: nonce++ }),
  ]);
  return expectSubstrateEvent(block, "referenda", "Submitted").data[0].toNumber();
}

// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
// Be careful to not reach the maximum number of proposals.
describeDevMoonbeam("Precompiles - Conviction Voting precompile", (context) => {
  let proposalIndex: number;
  beforeEach("create a proposal", async function () {
    proposalIndex = await createProposal(context);
  });

  it("should allow to vote yes for a proposal", async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1),
      })
    );

    // Verifies the EVM Side
    expectEVMResult(block.result.events, "Succeed");
    const { data } = await expectSubstrateEvent(block, "evm", "Log");
    const evmLog = CONVICTION_VOTING_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name, "Wrong event").to.equal("Voted");
    expect(evmLog.args.voter).to.equal(alith.address);
    expect(evmLog.args.pollIndex).to.equal(proposalIndex);
    expect(evmLog.args.aye).to.equal(true);
    expect(BigInt(evmLog.args.voteAmount)).to.equal(1n * 10n ** 18n);
    expect(evmLog.args.conviction).to.equal(1);

    // Verifies the Subsrtate side
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
  });

  it("should allow to vote no for a proposal", async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteNo(proposalIndex, 1n * 10n ** 18n, 1),
      })
    );
    // Verifies the EVM Side
    expectEVMResult(block.result.events, "Succeed");
    const { data } = await expectSubstrateEvent(block, "evm", "Log");
    const evmLog = CONVICTION_VOTING_INTERFACE.parseLog({
      topics: data[0].topics.map((t) => t.toHex()),
      data: data[0].data.toHex(),
    });
    expect(evmLog.name, "Wrong event").to.equal("Voted");
    expect(evmLog.args.voter).to.equal(alith.address);
    expect(evmLog.args.pollIndex).to.equal(proposalIndex);
    expect(evmLog.args.aye).to.equal(false);
    expect(BigInt(evmLog.args.voteAmount)).to.equal(1n * 10n ** 18n);
    expect(evmLog.args.conviction).to.equal(1);

    // Verifies the Subsrtate side
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(1n * 10n ** 18n);
  });

  it("should allow to replace yes by a no", async function () {
    const block1 = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1),
      })
    );
    expectEVMResult(block1.result.events, "Succeed");

    const block2 = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteNo(proposalIndex, 1n * 10n ** 18n, 1),
      })
    );
    expectEVMResult(block2.result.events, "Succeed");
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
    expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(1n * 10n ** 18n);
  });

  it("should fail to vote for the wrong proposal", async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteYes(999999, 1n * 10n ** 18n, 1),
      })
    );
    expectEVMResult(block.result.events, "Revert", "Reverted");
    const revertReason = await extractRevertReason(block.result.hash, context.ethers);
    expect(revertReason).to.contain("NotOngoing");
  });

  it("should fail to vote with the wrong conviction", async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 7),
      })
    );
    expectEVMResult(block.result.events, "Revert", "Reverted");
    const revertReason = await extractRevertReason(block.result.hash, context.ethers);
    expect(revertReason).to.contain("Must be an integer between 0 and 6 included");
  });
});

const CONVICTION_VALUES = [0n, 1n, 2n, 3n, 4n, 5n, 6n];
// Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
// Be careful to not reach the maximum number of proposals.
describeDevMoonbeam("Precompiles - Conviction", (context) => {
  let proposalIndex: number;
  beforeEach("create a proposal", async function () {
    proposalIndex = await createProposal(context);
  });

  for (const conviction of CONVICTION_VALUES) {
    it(`should allow to vote with confiction x${conviction}`, async function () {
      const block = await context.createBlock(
        createContractExecution(context, {
          contract: convictionVotingContract,
          contractCall: convictionVotingContract.methods.voteYes(
            proposalIndex,
            1n * 10n ** 18n,
            conviction
          ),
        })
      );
      expectEVMResult(block.result.events, "Succeed");

      // Verifies the Subsrtate side
      const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
      expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
        1n * 10n ** 17n * (conviction == 0n ? 1n : conviction * 10n)
      );
    });
  }
});

// Each test is instantiating a new proposal and a vote
// (Not ideal for isolation but easier to write)
// Be careful to not reach the maximum number of proposals.
describeDevMoonbeam("Precompiles - Conviction on Root Track", (context) => {
  let proposalIndex: number;
  beforeEach("create a proposal", async function () {
    proposalIndex = await createProposal(context);
    await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1),
      })
    );
    // Verifies the setup is correct
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
  });

  it(`should be removable`, async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVote(proposalIndex),
      })
    );
    expectEVMResult(block.result.events, "Succeed");

    // Verifies the Subsrtate side
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
  });

  it(`should be removable by specifying the track`, async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 0),
      })
    );
    expectEVMResult(block.result.events, "Succeed");

    // Verifies the Subsrtate side
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
  });
});

// Each test is instantiating a new proposal and a vote
// (Not ideal for isolation but easier to write)
// Be careful to not reach the maximum number of proposals.
describeDevMoonbeam("Precompiles - Conviction on General Admin Track", (context) => {
  let proposalIndex: number;
  beforeEach("create a proposal", async function () {
    proposalIndex = await createProposal(context, "generaladmin");
    await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1),
      })
    );
    // Verifies the setup is correct
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
  });

  it(`should be removable`, async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVote(proposalIndex),
      })
    );
    expectEVMResult(block.result.events, "Succeed");
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
  });

  it(`should be removable using self removeOtherVote`, async function () {
    // general_admin is track 2
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeOtherVote(
          alith.address,
          2,
          proposalIndex
        ),
      })
    );
    expectEVMResult(block.result.events, "Succeed");
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
  });

  it(`should be removable by specifying the track general_admin`, async function () {
    // general_admin is track 2
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 2),
      })
    );
    expectEVMResult(block.result.events, "Succeed");
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
  });

  it(`should not be removable by specifying the wrong track`, async function () {
    // general_admin is track 2
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 0),
      })
    );
    expectEVMResult(block.result.events, "Revert");
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
  });

  it(`should not be removable by someone else during voting time`, async function () {
    // general_admin is track 2
    const block = await context.createBlock(
      createContractExecution(
        context,
        {
          contract: convictionVotingContract,
          contractCall: convictionVotingContract.methods.removeOtherVote(
            alith.address,
            2,
            proposalIndex
          ),
        },
        { from: baltathar.address, privateKey: BALTATHAR_PRIVATE_KEY, gas: 1000000 }
      )
    );
    expectEVMResult(block.result.events, "Revert");
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
  });
});

describeDevMoonbeam("Precompiles - Ended proposal", (context) => {
  let proposalIndex: number;
  before("create a proposal and end it", async function () {
    // Whitelist caller is track 3
    proposalIndex = await createProposal(context, "whitelistedcaller");
    await expectOk(
      context.createBlock(context.polkadotApi.tx.referenda.placeDecisionDeposit(proposalIndex))
    );
    const alithAccount = await context.polkadotApi.query.system.account(alith.address);
    await expectOk(
      context.createBlock(
        createContractExecution(context, {
          contract: convictionVotingContract,
          contractCall: convictionVotingContract.methods.voteYes(
            proposalIndex,
            alithAccount.data.free.toBigInt() - 20n * 10n ** 18n,
            1
          ),
        })
      )
    );
    // 20 minutes jump
    await jumpBlocks(context, (20 * 60) / 12);

    // Verifies the setup is correct
    const referendum = await context.polkadotApi.query.referenda.referendumInfoFor(proposalIndex);
    expect(referendum.unwrap().isApproved).to.be.true;
  });

  // This and the next "it" and dependant on same state but this one is supposed to
  // revert and so not impact the proposal state
  it(`should failed to be removed without track info`, async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVote(proposalIndex),
      })
    );
    expectEVMResult(block.result.events, "Revert", "Reverted");
    expect(await extractRevertReason(block.result.hash, context.ethers)).to.contain("ClassNeeded");
  });

  it(`should be removable by specifying the track`, async function () {
    const block = await context.createBlock(
      createContractExecution(context, {
        contract: convictionVotingContract,
        contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 1),
      })
    );
    expectEVMResult(block.result.events, "Succeed");
  });
});
