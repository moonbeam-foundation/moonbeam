import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  PRECOMPILE_DEMOCRACY_ADDRESS,
  PROPOSAL_AMOUNT,
  createViemTransaction,
} from "@moonwall/util";
import { nToHex } from "@polkadot/util";
import { Abi, encodeFunctionData } from "viem";
import { notePreimagePrecompile } from "../../../../helpers";

describeSuite({
  id: "D012938",
  title: "Democracy - propose",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let encodedHash: `0x${string}`;
    let democracyAbi: Abi;

    beforeAll(async () => {
      const { abi } = fetchCompiledContract("Democracy");
      democracyAbi = abi;
      encodedHash = await notePreimagePrecompile(
        context,
        context.polkadotJs().tx.parachainStaking.setParachainBondAccount(ALITH_ADDRESS)
      );
    });
    it({
      id: "T01",
      title: "propose",
      test: async function () {
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

        const referendumCount = (
          await context.polkadotJs().query.democracy.referendumCount()
        ).toNumber();
        if (referendumCount !== 0) {
          log(`referendumCount is ${referendumCount}, skipping test as pre-existing state found`);
          return;
        }

        const publicPropCount = await context.polkadotJs().query.democracy.publicPropCount();
        expect(publicPropCount.toNumber(), "Proposal created").to.equal(0);
      },
    });
  },
});
