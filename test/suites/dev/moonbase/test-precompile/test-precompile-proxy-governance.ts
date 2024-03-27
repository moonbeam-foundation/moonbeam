import "@moonbeam-network/api-augment";
import {
  beforeAll,
  describeSuite,
  execCouncilProposal,
  execTechnicalCommitteeProposal,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  CONTRACT_PROXY_TYPE_GOVERNANCE,
  DOROTHY_ADDRESS,
  DOROTHY_PRIVATE_KEY,
  ETHAN_ADDRESS,
  ETHAN_PRIVATE_KEY,
  GLMR,
  PRECOMPILES,
  VOTE_AMOUNT,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../../helpers";

const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";

describeSuite({
  id: "D012960",
  title: "Proxing governance (through proxy precompile)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {
      log(`Disabled test D012761 (Gov V1)`);
      return;

      await execCouncilProposal(
        context,
        context.polkadotJs().tx.democracy.externalProposeMajority({
          Lookup: {
            hash: proposalHash,
            // Not testing scheduling, so length is moot
            len: 22,
          },
        })
      );
      await execTechnicalCommitteeProposal(
        context,
        context.polkadotJs().tx.democracy.fastTrack(proposalHash, 5, 0)
      );
    });

    it({
      id: "T01",
      title: "should be able to vote on behalf of the delegate account",
      test: async function () {
        log(`Disabled test D012761 (Gov V1)`);
        return;
        // Verify that one referundum is triggered
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        expect(referendumCount.toBigInt()).to.equal(1n);

        // Dorothy add proxy right to ethan for governance only
        const rawTx = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [ETHAN_ADDRESS, CONTRACT_PROXY_TYPE_GOVERNANCE, 0],
          privateKey: DOROTHY_PRIVATE_KEY,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTx);

        log("Dorothy add proxy right to ethan for governance only");
        expectEVMResult(result!.events, "Succeed");

        const dorothyPreBalance = (
          await context.polkadotJs().query.system.account(DOROTHY_ADDRESS)
        ).data.free.toBigInt();

        const { abi } = fetchCompiledContract("Democracy");
        const rawTx2 = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "proxy",
          args: [
            DOROTHY_ADDRESS,
            PRECOMPILES.Democracy,
            encodeFunctionData({
              abi,
              functionName: "standardVote",
              args: [0, true, VOTE_AMOUNT, 1],
            }),
          ],
          rawTxOnly: true,
          privateKey: ETHAN_PRIVATE_KEY,
        });
        const { result: result2 } = await context.createBlock(rawTx2);
        log("Ethan vote as Dorothy");
        expectEVMResult(result2!.events, "Succeed");

        // Verify that dorothy hasn't paid for the transaction but the vote locked her tokens
        const dorothyAccountData = await context.polkadotJs().query.system.account(DOROTHY_ADDRESS);
        expect(dorothyAccountData.data.free.toBigInt()).to.equal(dorothyPreBalance);
        expect(dorothyAccountData.data.frozen.toBigInt()).to.equal(VOTE_AMOUNT);

        // Verify that vote is registered
        const referendumInfoOf = (
          await context.polkadotJs().query.democracy.referendumInfoOf(0)
        ).unwrap();
        const onGoing = referendumInfoOf.asOngoing;

        expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(proposalHash);
        expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
        expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);
      },
    });
  },
});
