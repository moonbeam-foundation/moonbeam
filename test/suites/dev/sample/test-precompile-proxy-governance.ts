import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";

import { DOROTHY_ADDRESS, ETHAN_ADDRESS } from "../../util/accounts";
import {
  CONTRACT_PROXY_TYPE_GOVERNANCE,
  GLMR,
  VOTE_AMOUNT,
  PRECOMPILE_DEMOCRACY_ADDRESS,
  PRECOMPILE_PROXY_ADDRESS,
} from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { execCouncilProposal, execTechnicalCommitteeProposal } from "../../util/governance";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  createTransaction,
  DOROTHY_TRANSACTION_TEMPLATE,
  ETHAN_TRANSACTION_TEMPLATE,
} from "../../util/transactions";

const DEMOCRACY_CONTRACT = getCompiled("precompiles/pallet-democracy/Democracy");
const DEMOCRACY_INTERFACE = new ethers.utils.Interface(DEMOCRACY_CONTRACT.contract.abi);
const PROXY_CONTRACT_JSON = getCompiled("precompiles/proxy/Proxy");
const PROXY_INTERFACE = new ethers.utils.Interface(PROXY_CONTRACT_JSON.contract.abi);

const proposalHash = "0xf3d039875302d49d52fb1af6877a2c46bc55b004afb8130f94dd9d0489ca3185";

export async function getMappingInfo(
  context: DevTestContext,
  authorId: string
): Promise<{ account: string; deposit: BigInt }> {
  const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
  return null;
}

describeDevMoonbeam("Proxing governance (through proxy precompile)", (context) => {
  before("Create accounts and fast-tracking referundum", async () => {
    await execCouncilProposal(
      context,
      context.polkadotApi.tx.democracy.externalProposeMajority({
        Lookup: {
          hash: proposalHash,
          // this test does not test scheduling, therefore this lenght should not
          // matter
          len: 22,
        },
      } as any)
    );
    await execTechnicalCommitteeProposal(
      context,
      context.polkadotApi.tx.democracy.fastTrack(proposalHash, 5, 0)
    );
  });

  it("should be able to vote on behalf of the delegate account", async function () {
    // Verify that one referundum is triggered
    let referendumCount = await context.polkadotApi.query.democracy.referendumCount();
    expect(referendumCount.toBigInt()).to.equal(1n);

    // Dorothy add proxy right to ethan for governance only
    const {
      result: { events },
    } = await context.createBlock(
      createTransaction(context, {
        ...DOROTHY_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          ETHAN_ADDRESS,
          CONTRACT_PROXY_TYPE_GOVERNANCE,
          0,
        ]),
      })
    );
    expectEVMResult(events, "Succeed");

    const dorothyPreBalance = (
      await context.polkadotApi.query.system.account(DOROTHY_ADDRESS)
    ).data.free.toBigInt();

    // Ethan vote as Dorothy
    const {
      result: { events: events2 },
    } = await context.createBlock(
      createTransaction(context, {
        ...ETHAN_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("proxy", [
          DOROTHY_ADDRESS,
          PRECOMPILE_DEMOCRACY_ADDRESS,
          DEMOCRACY_INTERFACE.encodeFunctionData("standardVote", [0, true, VOTE_AMOUNT, 1]),
        ]),
      })
    );
    expectEVMResult(events2, "Succeed");

    // Verify that dorothy hasn't paid for the transaction but the vote locked her tokens
    let dorothyAccountData = await context.polkadotApi.query.system.account(DOROTHY_ADDRESS);
    expect(dorothyAccountData.data.free.toBigInt()).to.equal(dorothyPreBalance);
    expect(dorothyAccountData.data.miscFrozen.toBigInt()).to.equal(VOTE_AMOUNT);

    // Verify that vote is registered
    const referendumInfoOf = (
      await context.polkadotApi.query.democracy.referendumInfoOf(0)
    ).unwrap() as any;
    const onGoing = referendumInfoOf.asOngoing;

    expect(onGoing.proposal.asLookup.hash_.toHex()).to.equal(proposalHash);
    expect(onGoing.tally.ayes.toBigInt()).to.equal(10n * GLMR);
    expect(onGoing.tally.turnout.toBigInt()).to.equal(10n * GLMR);
  });
});
