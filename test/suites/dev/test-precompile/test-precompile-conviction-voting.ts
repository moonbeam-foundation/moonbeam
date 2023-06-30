import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  beforeAll,
  DevModeContext,
  beforeEach,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  PRECOMPILE_CONVICTION_VOTING_ADDRESS,
  alith,
  createRawTransaction,
} from "@moonwall/util";
import { expectSubstrateEvent } from "../../../helpers/expect.js";
import { Abi, decodeEventLog, encodeFunctionData } from "viem";
import { expectEVMResult, extractRevertReason } from "../../../helpers/eth-transactions.js";

async function createProposal(context: DevModeContext, track = "root") {
  let nonce = (await context.polkadotJs().rpc.system.accountNextIndex(ALITH_ADDRESS)).toNumber();
  const call = context.polkadotJs().tx.identity.setIdentity({ display: { raw: "Me" } });
  const block = await context.createBlock([
    await context
      .polkadotJs()
      .tx.preimage.notePreimage(call.toHex())
      .signAsync(alith, { nonce: nonce++ }),
    await context
      .polkadotJs()
      .tx.referenda.submit(
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
describeSuite({
  id: "D2529",
  title: "Precompiles - Conviction Voting precompile",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    let proposalIndex: number;
    let convictionVotingAbi: Abi;

    beforeAll(async function () {
      const { abi } = await fetchCompiledContract("ConvictionVoting");
      convictionVotingAbi = abi;
    });

    beforeEach(async function () {
      proposalIndex = await createProposal(context);
    });

    it({
      id: "T01",
      title: "should allow to vote yes for a proposal",
      test: async function () {
        const rawTx = await createRawTransaction(context, {
          to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
          data: encodeFunctionData({
            abi: convictionVotingAbi,
            functionName: "voteYes",
            args: [proposalIndex, 1n * 10n ** 18n, 1],
          }),
        });
        const block = await context.createBlock(rawTx);

        // Verifies the EVM Side
        expectEVMResult(block.result!.events, "Succeed");
        const { data } = expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: convictionVotingAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;

        expect(evmLog.eventName, "Wrong event").to.equal("Voted");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.aye).to.equal(true);
        expect(BigInt(evmLog.args.voteAmount)).to.equal(1n * 10n ** 18n);
        expect(evmLog.args.conviction).to.equal(1);

        // Verifies the Substrate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T02",
      title: "should allow to vote no for a proposal",
      test: async function () {
        const block = await context.createBlock(
          await createRawTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteNo",
              args: [proposalIndex, 1n * 10n ** 18n, 1],
            }),
          })
        );

        expectEVMResult(block.result!.events, "Succeed");
        const { data } = await expectSubstrateEvent(block, "evm", "Log");
        const evmLog = decodeEventLog({
          abi: convictionVotingAbi,
          topics: data[0].topics.map((t) => t.toHex()) as any,
          data: data[0].data.toHex(),
        }) as any;

        expect(evmLog.eventName, "Wrong event").to.equal("Voted");
        expect(evmLog.args.voter).to.equal(ALITH_ADDRESS);
        expect(evmLog.args.pollIndex).to.equal(proposalIndex);
        expect(evmLog.args.aye).to.equal(false);
        expect(BigInt(evmLog.args.voteAmount)).to.equal(1n * 10n ** 18n);
        expect(evmLog.args.conviction).to.equal(1);

        // Verifies the Subsrtate side
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T03",
      title: "should allow to replace yes by a no",
      test: async function () {
        const block1 = await context.createBlock(
          createRawTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteYes",
              args: [proposalIndex, 1n * 10n ** 18n, 1],
            }),
          })
        );
        expectEVMResult(block1.result!.events, "Succeed");

        const block2 = await context.createBlock(
          createRawTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteNo",
              args: [proposalIndex, 1n * 10n ** 18n, 1],
            }),
          })
        );
        expectEVMResult(block2.result!.events, "Succeed");
        const referendum = await context
          .polkadotJs()
          .query.referenda.referendumInfoFor(proposalIndex);
        expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
        expect(referendum.unwrap().asOngoing.tally.nays.toBigInt()).to.equal(1n * 10n ** 18n);
      },
    });

    it({
      id: "T04",
      title: "should fail to vote for the wrong proposal",
      test: async function () {
        const block = await context.createBlock(
          createRawTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteNo",
              args: [999999, 1n * 10n ** 18n, 1],
            }),
            skipEstimation: true,
          })
        );

        expectEVMResult(block.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(block.result!.hash, context.ethers());
        expect(revertReason).toContain("NotOngoing");
      },
    });

    it({
      id: "T05",
      title: "should fail to vote with the wrong conviction",
      test: async function () {
        const block = await context.createBlock(
          createRawTransaction(context, {
            to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
            data: encodeFunctionData({
              abi: convictionVotingAbi,
              functionName: "voteYes",
              args: [proposalIndex, 1n * 10n ** 18n, 7],
            }),
            skipEstimation: true,
          })
        );
        expectEVMResult(block.result!.events, "Revert", "Reverted");
        const revertReason = await extractRevertReason(block.result!.hash, context.ethers());
        expect(revertReason).to.contain("Must be an integer between 0 and 6 included");
      },
    });
  },
});

// const CONVICTION_VALUES = [0n, 1n, 2n, 3n, 4n, 5n, 6n];
// // Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
// // Be careful to not reach the maximum number of proposals.
// describeSuite("D2528Precompiles - Conviction", foundationMethods:"dev",tests:({it,log,context}) => {
//   let proposalIndex: number;
//   beforeEach("create a proposal", async function () {
//     proposalIndex = await createProposal(context);
//   });

//   for (const conviction of CONVICTION_VALUES) {
//     it(`should allow to vote with confiction x${conviction}`, async function () {
//       const block = await context.createBlock(
//         createContractExecution(context, {
//           contract: convictionVotingContract,
//           contractCall: convictionVotingContract.methods.voteYes(
//             proposalIndex,
//             1n * 10n ** 18n,
//             conviction
//           ),
//         })
//       );
//       expectEVMResult(block.result.events, "Succeed");

//       // Verifies the Subsrtate side
//       const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//       expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
//         1n * 10n ** 17n * (conviction == 0n ? 1n : conviction * 10n)
//       );
//     });
//   }
// });

// // Each test is instantiating a new proposal and a vote
// // (Not ideal for isolation but easier to write)
// // Be careful to not reach the maximum number of proposals.
// describeSuite("D2528Precompiles - Conviction on Root Track", foundationMethods:"dev",tests:({it,log,context}) => {
//   let proposalIndex: number;
//   beforeEach("create a proposal", async function () {
//     proposalIndex = await createProposal(context);
//     await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1),
//       })
//     );
//     // Verifies the setup is correct
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
//   });

//   it(`should be removable`, async function () {
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVote(proposalIndex),
//       })
//     );
//     expectEVMResult(block.result.events, "Succeed");

//     // Verifies the Subsrtate side
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
//   });

//   it(`should be removable by specifying the track`, async function () {
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 0),
//       })
//     );
//     expectEVMResult(block.result.events, "Succeed");

//     // Verifies the Subsrtate side
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
//   });
// });

// // Each test is instantiating a new proposal and a vote
// // (Not ideal for isolation but easier to write)
// // Be careful to not reach the maximum number of proposals.
// describeSuite("D2528Precompiles - Conviction on General Admin Track", foundationMethods:"dev",tests:({it,log,context}) => {
//   let proposalIndex: number;
//   beforeEach("create a proposal", async function () {
//     proposalIndex = await createProposal(context, "generaladmin");
//     await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1),
//       })
//     );
//     // Verifies the setup is correct
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
//   });

//   it(`should be removable`, async function () {
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVote(proposalIndex),
//       })
//     );
//     expectEVMResult(block.result.events, "Succeed");
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
//   });

//   it(`should be removable using self removeOtherVote`, async function () {
//     // general_admin is track 2
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeOtherVote(
//           ALITH_ADDRESS,
//           2,
//           proposalIndex
//         ),
//       })
//     );
//     expectEVMResult(block.result.events, "Succeed");
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
//   });

//   it(`should be removable by specifying the track general_admin`, async function () {
//     // general_admin is track 2
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 2),
//       })
//     );
//     expectEVMResult(block.result.events, "Succeed");
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(0n);
//   });

//   it(`should not be removable by specifying the wrong track`, async function () {
//     // general_admin is track 2
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 0),
//       })
//     );
//     expectEVMResult(block.result.events, "Revert");
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
//   });

//   it(`should not be removable by someone else during voting time`, async function () {
//     // general_admin is track 2
//     const block = await context.createBlock(
//       createContractExecution(
//         context,
//         {
//           contract: convictionVotingContract,
//           contractCall: convictionVotingContract.methods.removeOtherVote(
//             ALITH_ADDRESS,
//             2,
//             proposalIndex
//           ),
//         },
//         { from: baltathar.address, privateKey: BALTATHAR_PRIVATE_KEY, gas: 1000000 }
//       )
//     );
//     expectEVMResult(block.result.events, "Revert");
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(1n * 10n ** 18n);
//   });
// });

// describeSuite("D2528Precompiles - Ended proposal", foundationMethods:"dev",tests:({it,log,context}) => {
//   let proposalIndex: number;
//   before("create a proposal and end it", async function () {
//     // Whitelist caller is track 3
//     proposalIndex = await createProposal(context, "whitelistedcaller");
//     await expectOk(
//       context.createBlock(context.polkadotJs().tx.referenda.placeDecisionDeposit(proposalIndex))
//     );
//     const alithAccount = await context.polkadotJs().query.system.account(ALITH_ADDRESS);
//     await expectOk(
//       context.createBlock(
//         createContractExecution(context, {
//           contract: convictionVotingContract,
//           contractCall: convictionVotingContract.methods.voteYes(
//             proposalIndex,
//             alithAccount.data.free.toBigInt() - 20n * 10n ** 18n,
//             1
//           ),
//         })
//       )
//     );
//     // 20 minutes jump
//     await jumpBlocks(context, (20 * 60) / 12);

//     // Verifies the setup is correct
//     const referendum = await context.polkadotJs().query.referenda.referendumInfoFor(proposalIndex);
//     expect(referendum.unwrap().isApproved).to.be.true;
//   });

//   // This and the next "it" and dependant on same state but this one is supposed to
//   // revert and so not impact the proposal state
//   it(`should failed to be removed without track info`, async function () {
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVote(proposalIndex),
//       })
//     );
//     expectEVMResult(block.result.events, "Revert", "Reverted");
//     expect(await extractRevertReason(block.result.hash, context.ethers)).to.contain("ClassNeeded");
//   });

//   it(`should be removable by specifying the track`, async function () {
//     const block = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.removeVoteForTrack(proposalIndex, 1),
//       })
//     );
//     expectEVMResult(block.result.events, "Succeed");
//   });
// });

// // Each test is instantiating a new proposal (Not ideal for isolation but easier to write)
// // Be careful to not reach the maximum number of proposals.
// describeSuite("D2528Precompiles - ClassLocksFor & VotingFor", foundationMethods:"dev",tests:({it,log,context}) => {
//   let proposalIndex: number;
//   before("create a proposal", async function () {
//     proposalIndex = await createProposal(context);

//     const blockAlith_1 = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 1n * 10n ** 18n, 1n),
//       })
//     );
//     expectEVMResult(blockAlith_1.result.events, "Succeed");

//     const blockAlith_2 = await context.createBlock(
//       createContractExecution(context, {
//         contract: convictionVotingContract,
//         contractCall: convictionVotingContract.methods.voteYes(proposalIndex, 2n * 10n ** 18n, 2n),
//       })
//     );
//     expectEVMResult(blockAlith_2.result.events, "Succeed");

//     const blockBaltathar = await context.createBlock(
//       createContractExecution(
//         context,
//         {
//           contract: convictionVotingContract,
//           contractCall: convictionVotingContract.methods.voteYes(
//             proposalIndex,
//             3n * 10n ** 18n,
//             3n
//           ),
//         },
//         {
//           from: baltathar.address,
//           privateKey: BALTATHAR_PRIVATE_KEY,
//         }
//       )
//     );
//     expectEVMResult(blockBaltathar.result.events, "Succeed");
//   });

//   it("should return classLocksFor alith", async function () {
//     const { result } = await web3EthCall(context.web3, {
//       to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
//       data: CONVICTION_VOTING_INTERFACE.encodeFunctionData("classLocksFor", [ALITH_ADDRESS]),
//     });

//     const classLocksFor = CONVICTION_VOTING_INTERFACE.decodeFunctionResult(
//       "classLocksFor(address)",
//       result
//     )[0];

//     expect(classLocksFor.length).to.equal(1);
//     expect(classLocksFor[0].trackId).to.equal(0);
//     expect(classLocksFor[0].amount.toString()).to.equal((2n * 10n ** 18n).toString());
//   });

//   it("should return classLocksFor baltathar", async function () {
//     const { result } = await web3EthCall(context.web3, {
//       to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
//       data: CONVICTION_VOTING_INTERFACE.encodeFunctionData("classLocksFor", [baltathar.address]),
//     });

//     const classLocksFor = CONVICTION_VOTING_INTERFACE.decodeFunctionResult(
//       "classLocksFor(address)",
//       result
//     )[0];

//     expect(classLocksFor.length).to.equal(1);
//     expect(classLocksFor[0].trackId).to.equal(0);
//     expect(classLocksFor[0].amount.toString()).to.equal((3n * 10n ** 18n).toString());
//   });

//   it("should return votingFor alith", async function () {
//     const { result } = await web3EthCall(context.web3, {
//       to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
//       data: CONVICTION_VOTING_INTERFACE.encodeFunctionData("votingFor", [
//         ALITH_ADDRESS,
//         proposalIndex,
//       ]),
//     });

//     const votingFor = CONVICTION_VOTING_INTERFACE.decodeFunctionResult(
//       "votingFor(address,uint16)",
//       result
//     )[0];

//     expect(votingFor.casting.votes).to.have.lengthOf(1);
//     expect(votingFor.casting.votes[0].pollIndex.toString()).to.equal("0");
//     expect(votingFor.casting.votes[0].accountVote.isStandard).to.be.true;
//     expect(votingFor.casting.votes[0].accountVote.isSplit).to.be.false;
//     expect(votingFor.casting.votes[0].accountVote.isSplitAbstain).to.be.false;
//     expect(votingFor.casting.votes[0].accountVote.standard.vote.aye).to.be.true;
//     expect(votingFor.casting.votes[0].accountVote.standard.vote.conviction).to.equal(2);
//     expect(votingFor.casting.votes[0].accountVote.standard.balance.toString()).to.equal(
//       (2n * 10n ** 18n).toString()
//     );
//     expect(votingFor.casting.prior.balance.toString()).to.equal("0");
//     expect(votingFor.casting.delegations.votes.toString()).to.equal("0");
//     expect(votingFor.casting.delegations.capital.toString()).to.equal("0");
//     expect(votingFor.isCasting).to.be.true;
//     expect(votingFor.isDelegating).to.be.false;
//   });

//   it("should return votingFor baltathar", async function () {
//     const { result } = await web3EthCall(context.web3, {
//       to: PRECOMPILE_CONVICTION_VOTING_ADDRESS,
//       data: CONVICTION_VOTING_INTERFACE.encodeFunctionData("votingFor", [
//         baltathar.address,
//         proposalIndex,
//       ]),
//     });

//     const votingFor = CONVICTION_VOTING_INTERFACE.decodeFunctionResult(
//       "votingFor(address,uint16)",
//       result
//     )[0];

//     expect(votingFor.casting.votes).to.have.lengthOf(1);
//     expect(votingFor.casting.votes[0].pollIndex.toString()).to.equal("0");
//     expect(votingFor.casting.votes[0].accountVote.isStandard).to.be.true;
//     expect(votingFor.casting.votes[0].accountVote.isSplit).to.be.false;
//     expect(votingFor.casting.votes[0].accountVote.isSplitAbstain).to.be.false;
//     expect(votingFor.casting.votes[0].accountVote.standard.vote.aye).to.be.true;
//     expect(votingFor.casting.votes[0].accountVote.standard.vote.conviction).to.equal(3);
//     expect(votingFor.casting.votes[0].accountVote.standard.balance.toString()).to.equal(
//       (3n * 10n ** 18n).toString()
//     );
//     expect(votingFor.casting.prior.balance.toString()).to.equal("0");
//     expect(votingFor.casting.delegations.votes.toString()).to.equal("0");
//     expect(votingFor.casting.delegations.capital.toString()).to.equal("0");
//     expect(votingFor.isCasting).to.be.true;
//     expect(votingFor.isDelegating).to.be.false;
//   });
// });
