import "@moonbeam-network/api-augment";
import type { DevModeContext } from "moonwall";
import type { FrameSupportPreimagesBounded } from "@polkadot/types/lookup";
import type { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import type { KeyringPair } from "@polkadot/keyring/types";
import { notePreimage, execOpenTechCommitteeProposal, faith, alith } from "moonwall";
import chalk from "chalk";
import Debugger from "debug";

const log = Debugger("test:referenda");

/**
 * Sets up a whitelisted referendum without placing decision deposit or voting.
 * Used by tests that need to manually control the referendum lifecycle.
 *
 * This differs from moonwall's `whiteListedTrack` which executes the full
 * referendum lifecycle including voting and fast-forwarding.
 */
export const whiteListTrackNoSend = async <
  Call extends SubmittableExtrinsic<ApiType>,
  ApiType extends ApiTypes,
>(
  context: DevModeContext,
  proposal: string | Call
): Promise<{ whitelistedHash: string }> => {
  const proposalHash =
    typeof proposal === "string" ? proposal : await notePreimage(context, proposal);

  // Construct dispatchWhiteListed call
  const proposalLen = (await context.pjsApi.query.preimage.requestStatusFor(proposalHash)).unwrap()
    .asUnrequested.len;

  const dispatchWLCall = context.pjsApi.tx.whitelist.dispatchWhitelistedCall(
    proposalHash,
    proposalLen,
    { refTime: 2_000_000_000, proofSize: 100_000 }
  );

  // Note preimage of dispatchWhitelistedCall
  const wLPreimage = await notePreimage(context, dispatchWLCall);
  const wLPreimageLen = dispatchWLCall.encodedLength - 2;

  // Submit openGov proposal (without placing decision deposit)
  const openGovProposal = await context.pjsApi.tx.referenda
    .submit(
      { Origins: { whitelistedcaller: "WhitelistedCaller" } },
      { Lookup: { hash: wLPreimage, len: wLPreimageLen } },
      { After: { After: 0 } }
    )
    .signAsync(faith);

  await context.createBlock(openGovProposal);

  // Whitelist the original call via openTechCommittee
  const whitelistCall = context.pjsApi.tx.whitelist.whitelistCall(proposalHash);
  await execOpenTechCommitteeProposal(context, whitelistCall);

  return { whitelistedHash: wLPreimage };
};

export interface ForceReducedReferendaExecutionOptions {
  forceTally?: boolean; // Will force tally to match total issuance
}

/**
 * @description Force a referendum to be executed in the next blocks by
 *              changing the referenda and scheduler storage data
 *              This function will create few blocks to ensure the referendum is executed.
 */
export const forceReducedReferendaExecution = async (
  context: DevModeContext,
  proposalIndex: number,
  options: ForceReducedReferendaExecutionOptions = {}
) => {
  const forceTally = options?.forceTally || false;
  const api = context.polkadotJs();

  log(
    `[#${chalk.green((await api.rpc.chain.getHeader()).number.toNumber())}]: Referedum ${chalk.red(
      proposalIndex
    )}`
  );

  const referendumData = await api.query.referenda.referendumInfoFor(proposalIndex);
  const referendumKey = api.query.referenda.referendumInfoFor.key(proposalIndex);
  if (!referendumData.isSome) {
    throw new Error(`Referendum ${proposalIndex} not found`);
  }
  const referendumInfo = referendumData.unwrap();
  if (!referendumInfo.isOngoing) {
    throw new Error(`Referendum ${proposalIndex} is not ongoing`);
  }

  const ongoingData = referendumInfo.asOngoing;
  const ongoingJson = ongoingData.toJSON();
  const callHash = ongoingData.proposal.asLookup.toHex();
  const proposalBlockTarget = (await api.rpc.chain.getHeader()).number.toNumber();
  const fastProposalData: any = {
    ongoing: {
      ...ongoingJson,
      enactment: { after: 0 },
      deciding: {
        since: proposalBlockTarget - 1,
        confirming: proposalBlockTarget - 1,
      },
      alarm: [proposalBlockTarget + 1, [proposalBlockTarget + 1, 0]],
    },
  };
  if (forceTally) {
    const totalIssuance = (await api.query.balances.totalIssuance()).toBigInt();
    fastProposalData.tally = {
      ayes: totalIssuance - 1n,
      nays: 0,
      support: totalIssuance - 1n,
    };
  }

  let fastProposal: any;
  try {
    fastProposal = api.registry.createType(
      "Option<PalletReferendaReferendumInfo>",
      fastProposalData
    );
  } catch {
    fastProposal = api.registry.createType(
      "Option<PalletReferendaReferendumInfoConvictionVotingTally>",
      fastProposalData
    );
  }

  log(
    `${chalk.blue("SetStorage")} Fast Proposal: ${chalk.red(
      proposalIndex.toString()
    )} referendumKey ${referendumKey}`
  );

  await context.createBlock(
    api.tx.sudo.sudo(api.tx.system.setStorage([[referendumKey, fastProposal.toHex()]])),
    { allowFailures: false }
  );

  // Fast forward the nudge referendum to the next block to get the refendum to be scheduled
  log(
    `${chalk.yellow("Rescheduling")} ${chalk.red("scheduler.nudgeReferendum")} to #${chalk.green(
      (await api.rpc.chain.getHeader()).number.toNumber() + 2
    )}`
  );

  await moveScheduledCallTo(context, 2, (call) => {
    if (!call.isInline) {
      return false;
    }
    const callData = api.createType("Call", call.asInline.toHex());
    return (
      callData.method === "nudgeReferendum" &&
      (callData.args[0] as any).toNumber() === proposalIndex
    );
  });

  log(
    `${chalk.yellow("Fast forward")} ${chalk.green(1)} to #${chalk.green(
      (await api.rpc.chain.getHeader()).number.toNumber() + 1
    )}`
  );
  await context.createBlock();

  // Fast forward the scheduled proposal
  log(
    `${chalk.yellow("Rescheduling")} proposal ${chalk.red(proposalIndex)} to #${chalk.green(
      (await api.rpc.chain.getHeader()).number.toNumber() + 2
    )}`
  );
  await moveScheduledCallTo(
    context,
    2,
    (call) => call.isLookup && call.asLookup.toHex() === callHash
  );

  log(
    `${chalk.yellow("Fast forward")} ${chalk.green(1)} to #${chalk.green(
      (await api.rpc.chain.getHeader()).number.toNumber() + 1
    )}`
  );
  await context.createBlock();
};

async function moveScheduledCallTo(
  context: DevModeContext,
  blockCounts: number,
  verifier: (call: FrameSupportPreimagesBounded) => boolean
) {
  const api = context.polkadotJs();
  const blockNumber = (await api.rpc.chain.getHeader()).number.toNumber();
  // Fast forward the nudge referendum to the next block to get the refendum to be scheduled
  const agenda = await api.query.scheduler.agenda.entries();
  const storages: [string, string][] = [];
  const deleteStorages: string[] = [];
  for (const agendaEntry of agenda) {
    for (const scheduledEntry of agendaEntry[1]) {
      if (scheduledEntry.isSome && verifier(scheduledEntry.unwrap().call)) {
        log(`${chalk.blue("SetStorage")} scheduler.agenda`);
        deleteStorages.push(agendaEntry[0].toHex());
        storages.push([
          await api.query.scheduler.agenda.key(blockNumber + blockCounts),
          agendaEntry[1].toHex(),
        ]);
        if (scheduledEntry.unwrap().maybeId.isSome) {
          const id = scheduledEntry.unwrap().maybeId.unwrap().toHex();
          const lookup = await api.query.scheduler.lookup(id);
          log(
            `Checking lookup ${scheduledEntry.unwrap().maybeId.unwrap().toHex()}: ${lookup.isSome}`
          );
          if (lookup.isSome) {
            const fastLookup = api.registry.createType("Option<(u32,u32)>", [
              blockNumber + blockCounts,
              0,
            ]);
            log(`Updated lookup to ${fastLookup.toJSON()}`);
          }
        }
      }
    }
  }

  if (storages.length === 0) {
    throw new Error("No scheduled call found");
  }
  await context.createBlock(
    api.tx.sudo.sudo(
      api.tx.utility.batchAll([
        api.tx.system.setStorage(storages),
        api.tx.system.killStorage(deleteStorages),
      ])
    ),
    {
      allowFailures: false,
    }
  );
}

/**
 * Maximizes conviction voting for voters on an ongoing referendum.
 * Uses each voter's full free balance minus estimated fees with maximum conviction (6x).
 */
export const maximizeConvictionVotingOf = async (
  context: DevModeContext,
  voters: KeyringPair[],
  refIndex: number
): Promise<void> => {
  const fee = (
    await context
      .polkadotJs()
      .tx.convictionVoting.vote(refIndex, {
        Standard: {
          vote: { aye: true, conviction: "Locked6x" },
          balance: (await context.polkadotJs().query.system.account(alith.address)).data.free,
        },
      })
      .paymentInfo(alith)
  ).partialFee;

  await context.createBlock(
    voters.map(async (voter) =>
      context
        .polkadotJs()
        .tx.convictionVoting.vote(refIndex, {
          Standard: {
            vote: { aye: true, conviction: "Locked6x" },
            balance: (await context.polkadotJs().query.system.account(voter.address)).data.free.sub(
              fee
            ),
          },
        })
        .signAsync(voter)
    )
  );
};

/**
 * Fast-forwards to the next scheduled event in the scheduler agenda.
 * Useful for advancing through referendum lifecycle phases (preparation, decision, enactment).
 */
export const fastFowardToNextEvent = async (context: DevModeContext): Promise<void> => {
  const [entry] = await context.pjsApi.query.scheduler.agenda.entries();
  const [key] = entry;

  if (key.isEmpty) {
    throw new Error("No items in scheduler.agenda");
  }

  const decodedKey = key.toHuman() as string[];
  const desiredHeight = Number(decodedKey[0].replace(/,/g, ""));
  const currentHeight = (await context.pjsApi.rpc.chain.getHeader()).number.toNumber();

  log(
    `Fast forwarding from block ${currentHeight} to ${desiredHeight} (${desiredHeight - currentHeight + 1} blocks)`
  );

  await context.jumpBlocks?.(desiredHeight - currentHeight + 1);
};
