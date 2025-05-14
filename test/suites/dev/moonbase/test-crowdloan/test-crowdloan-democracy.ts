import "@moonbeam-network/api-augment";
import {
  describeSuite, DevModeContext, execOpenTechCommitteeProposal,
  expect,
  fastFowardToNextEvent, filterAndApply,
  maximizeConvictionVotingOf,
  notePreimage,
} from "@moonwall/cli";
import {
  baltathar,
  DEFAULT_GENESIS_BALANCE,
  ethan,
  faith,
  GLMR,
  GOLIATH_ADDRESS
} from "@moonwall/util";
import type { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import {
  getAccountPayable,
  RELAYCHAIN_ARBITRARY_ADDRESS_1,
  RELAYCHAIN_ARBITRARY_ADDRESS_2,
  VESTING_PERIOD,
} from "../../../../helpers";
import {ApiTypes} from "@polkadot/api/types";

export const whiteListTrackNoSend = async <
  Call extends SubmittableExtrinsic
>(
  context: DevModeContext,
  proposal: string | Call
) => {
  const proposalHash =
    typeof proposal === "string" ? proposal : await notePreimage(context, proposal);

  const proposalLen = (await context.pjsApi.query.preimage.requestStatusFor(proposalHash)).unwrap()
    .asUnrequested.len;
  const dispatchWLCall = context.pjsApi.tx.whitelist.dispatchWhitelistedCall(
    proposalHash,
    proposalLen,
    {
      refTime: 2_000_000_000,
      proofSize: 4_000_000,
    }
  );

  const wLPreimage = await notePreimage(context, dispatchWLCall);
  const wLPreimageLen = dispatchWLCall.encodedLength - 2;
  console.log(
    `📝 DispatchWhitelistedCall preimage noted: ${wLPreimage.slice(0, 6)}...${wLPreimage.slice(
      -4
    )}, len: ${wLPreimageLen}`
  );

  const openGovProposal = await context.pjsApi.tx.referenda
    .submit(
      {
        Origins: { whitelistedcaller: "WhitelistedCaller" },
      },
      { Lookup: { hash: wLPreimage, len: wLPreimageLen } },
      { After: { After: 0 } }
    )
    .signAsync(faith);
  const { result } = await context.createBlock(openGovProposal);

  if (!result?.events) {
    throw new Error("No events in block");
  }

  let proposalId: number | undefined;
  filterAndApply(result.events, "referenda", ["Submitted"], (found) => {
    proposalId = (found.event as any).data.index.toNumber();
  });

  if (typeof proposalId === "undefined") {
    throw new Error("No proposal id found");
  }

  console.log(`🏛️ Referendum submitted with proposal id: ${proposalId}`);
  await context.createBlock(context.pjsApi.tx.referenda.placeDecisionDeposit(proposalId));

  const whitelistCall = context.pjsApi.tx.whitelist.whitelistCall(proposalHash);
  await execOpenTechCommitteeProposal(context, whitelistCall);
  return { proposalHash, whitelistedHash: wLPreimage };
};

describeSuite({
  id: "D010805",
  title: "Crowdloan - Democracy",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be able to initialize through democracy",
      test: async () => {
        const initializeRewardVec = context.polkadotJs().tx.crowdloanRewards.initializeRewardVec([
            [RELAYCHAIN_ARBITRARY_ADDRESS_1, GOLIATH_ADDRESS, 1_500_000n * GLMR],
            [RELAYCHAIN_ARBITRARY_ADDRESS_2, null, 1_500_000n * GLMR],
        ]);

        const initBlock = await context.polkadotJs().query.crowdloanRewards.initRelayBlock();
        const completeInitialization = context
            .polkadotJs()
            .tx.crowdloanRewards.completeInitialization(initBlock.toBigInt() + VESTING_PERIOD);

        await whiteListTrackNoSend(context, initializeRewardVec);
        await whiteListTrackNoSend(context, completeInitialization);

        await maximizeConvictionVotingOf(context, [ethan], 0);
        await maximizeConvictionVotingOf(context, [baltathar], 1);
        await context.createBlock();

        await fastFowardToNextEvent(context); // ⏩️ until preparation done
        await fastFowardToNextEvent(context); // ⏩️ until proposal confirmed
        await fastFowardToNextEvent(context); // ⏩️ until proposal enacted

        await fastFowardToNextEvent(context); // ⏩️ until proposal 2 is enacted

        const isInitialized = await context.polkadotJs().query.crowdloanRewards.initialized();
        expect(isInitialized.toHuman()).to.be.true;

        const reward_info_associated = await getAccountPayable(context, GOLIATH_ADDRESS);

        const reward_info_unassociated = (
          await context
            .polkadotJs()
            .query.crowdloanRewards.unassociatedContributions(RELAYCHAIN_ARBITRARY_ADDRESS_2)
        ).unwrap();

        // Check payments
        expect(reward_info_associated!.totalReward.toBigInt()).toBe(1_500_000n * GLMR);
        expect(reward_info_associated!.claimedReward.toBigInt()).toBe(450_000n * GLMR);
        expect(reward_info_unassociated.totalReward.toBigInt()).toBe(1_500_000n * GLMR);
        expect(reward_info_unassociated.claimedReward.toBigInt()).toBe(0n);

        // check balances
        const account = await context.polkadotJs().query.system.account(GOLIATH_ADDRESS);
        expect(account.data.free.toBigInt() - DEFAULT_GENESIS_BALANCE).toBe(
          reward_info_associated!.claimedReward.toBigInt()
        );
      },
    });
  },
});
