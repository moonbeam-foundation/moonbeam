import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  PRECOMPILE_DEMOCRACY_ADDRESS,
  PROPOSAL_AMOUNT,
  createViemTransaction,
} from "@moonwall/util";
import { nToHex } from "@polkadot/util";
import { Abi, encodeFunctionData, numberToHex } from "viem";
import { notePreimagePrecompile } from "../../../helpers/precompiles.js";

describeSuite({
  id: "D2532",
  title: "Democracy - second proposal",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let encodedHash: `0x${string}`;
    let launchPeriod: number;
    let democracyAbi: Abi;

    beforeAll(async () => {
      const { abi } = fetchCompiledContract("Democracy");
      democracyAbi = abi;
      launchPeriod = context.polkadotJs().consts.democracy.launchPeriod.toNumber();
      log(`launchPeriod: ${launchPeriod}`);

      encodedHash = await notePreimagePrecompile(
        context,
        context.polkadotJs().tx.parachainStaking.setParachainBondAccount(ALITH_ADDRESS)
      );

      await context.createBlock(
        createViemTransaction(context, {
          to: PRECOMPILE_DEMOCRACY_ADDRESS,
          gas: 2_000_000n,
          data: encodeFunctionData({
            abi: democracyAbi,
            functionName: "propose",
            args: [encodedHash, nToHex(PROPOSAL_AMOUNT)],
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
      title: "second proposal",
      test: async function () {
        const publicProps = await context.polkadotJs().query.democracy.publicProps();
        expect(publicProps[0][1].asLookup.hash_.toString()).to.equal(encodedHash);
        expect(publicProps[0][2].toString()).to.equal(ALITH_ADDRESS);

        const depositOf = await context.polkadotJs().query.democracy.depositOf(0);
        expect(depositOf.unwrap()[1].toBigInt()).to.equal(1_000_000_000_000_000_000_000n);
        expect(depositOf.unwrap()[0][1].toString()).to.equal(BALTATHAR_ADDRESS);
      },
    });

    it({
      id: "T02",
      title: "check referendum is up",
      timeout: 1000000,
      test: async function () {
        // After Launchperiod elapses, turn the proposal into a referendum
        // launchPeriod minus the 3 blocks that already elapsed
        log(`Creating ${launchPeriod - 3} blocks so that new referendum can be raised`);
        for (let i = 0; i < launchPeriod - 3; i++) {
          await context.createBlock();
        }
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        expect(referendumCount.toNumber()).to.equal(1);

        const publicPropCount = await context.polkadotJs().query.democracy.publicPropCount();
        expect(publicPropCount.toNumber()).to.equal(1);

        const referendumInfoOf = await context.polkadotJs().query.democracy.referendumInfoOf(0);
        expect(referendumInfoOf.unwrap().asOngoing.proposal.asLookup.hash_.toString()).to.equal(
          encodedHash
        );
      },
    });
  },
});
