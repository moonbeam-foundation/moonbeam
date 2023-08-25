import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_PRIVATE_KEY,
  PRECOMPILE_DEMOCRACY_ADDRESS,
  PROPOSAL_AMOUNT,
  VOTE_AMOUNT,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { Abi, encodeFunctionData, numberToHex } from "viem";
import { notePreimagePrecompile } from "../../../helpers/precompiles.js";

describeSuite({
  id: "D2533",
  title: "Democracy - vote on referendum",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let encodedHash: `0x${string}`;
    let enactmentPeriod: bigint;
    let democracyAbi: Abi;
    let launchPeriod: number;

    beforeAll(async () => {
      enactmentPeriod = context.polkadotJs().consts.democracy.enactmentPeriod.toBigInt();
      launchPeriod = context.polkadotJs().consts.democracy.launchPeriod.toNumber();
      const { abi } = fetchCompiledContract("Democracy");
      democracyAbi = abi;

      encodedHash = await notePreimagePrecompile(
        context,
        context.polkadotJs().tx.parachainStaking.setParachainBondAccount(ALITH_ADDRESS)
      );

      await context.createBlock(
        createViemTransaction(context, {
          to: PRECOMPILE_DEMOCRACY_ADDRESS,
          privateKey: ALITH_PRIVATE_KEY,
          gas: 2_000_000n,
          data: encodeFunctionData({
            abi: democracyAbi,
            functionName: "propose",
            args: [encodedHash, numberToHex(PROPOSAL_AMOUNT)],
          }),
        })
      );

      await context.createBlock(
        createViemTransaction(context, {
          to: PRECOMPILE_DEMOCRACY_ADDRESS,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 2_000_000n,
          data: encodeFunctionData({
            abi: democracyAbi,
            functionName: "second",
            args: [numberToHex(0), numberToHex(1000)],
          }),
        })
      );
    });

    it({
      id: "T01",
      title: "vote",
      timeout: 2000000,
      test: async function () {
        // After Launchperiod elapses, turn the proposal into a referendum
        // launchPeriod minus the 3 blocks that already elapsed
        log(`Creating ${launchPeriod - 3} blocks so that new referendum can be raised`);
        for (let i = 0; i < launchPeriod - 3; i++) {
          await context.createBlock();
        }

        await context.createBlock(
          createViemTransaction(context, {
            to: PRECOMPILE_DEMOCRACY_ADDRESS,
            privateKey: ALITH_PRIVATE_KEY,
            gas: 2_000_000n,
            data: encodeFunctionData({
              abi: democracyAbi,
              functionName: "standardVote",
              args: [numberToHex(0), "0x01", numberToHex(VOTE_AMOUNT), numberToHex(1)],
            }),
          })
        );

        const referendumInfoOf = await context.polkadotJs().query.democracy.referendumInfoOf(0);
        expect(referendumInfoOf.unwrap().asOngoing.proposal.asLookup.hash_.toHex()).to.equal(
          encodedHash
        );
        expect(referendumInfoOf.unwrap().asOngoing.delay.toBigInt()).to.equal(enactmentPeriod);
        expect(referendumInfoOf.unwrap().asOngoing.tally.ayes.toBigInt()).to.equal(
          10_000_000_000_000_000_000n
        );
        expect(referendumInfoOf.unwrap().asOngoing.tally.turnout.toBigInt()).to.equal(
          10_000_000_000_000_000_000n
        );

        const referendumHex = referendumInfoOf.toHex();

        // Instead of waiting votePeriod + enactmentPeriod (which is very long) we hack
        // the referendum to be shorter
        const blockNumber = (await context.polkadotJs().rpc.chain.getHeader()).number.toBigInt();

        const newEndBlock = context.polkadotJs().registry.createType("u32", blockNumber + 2n);
        const delayAmount = referendumInfoOf.unwrap().asOngoing.delay.toBigInt() - enactmentPeriod;
        const delay = context
          .polkadotJs()
          .registry.createType("u32", delayAmount < 0n ? 0n : delayAmount);

        // taking same referendum with different end & delay
        const modReferendum = `0x00${newEndBlock.toHex(true).slice(2)}${referendumHex.slice(
          12,
          88
        )}${delay.toHex(true).slice(2)}${referendumHex.slice(96)}`;

        // Changing storage for the referendum using sudo
        await context
          .polkadotJs()
          .tx.sudo.sudo(
            context
              .polkadotJs()
              .tx.system.setStorage([
                [
                  context.polkadotJs().query.democracy.referendumInfoOf.key(0).toString(),
                  modReferendum,
                ],
              ])
          )
          .signAndSend(alith);

        await context.createBlock();

        // Waiting extra blocks for the vote to finish
        for (let i = 0; i < 10; i++) {
          await context.createBlock();
        }

        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();

        const referendumDone = await context.polkadotJs().query.democracy.referendumInfoOf(0);

        expect(referendumDone.unwrap().isFinished).to.be.true;
        expect(referendumDone.unwrap().asFinished.approved.isTrue).to.be.true;
        expect(parachainBondInfo.account.toString()).to.equal(ALITH_ADDRESS);
      },
    });
  },
});
