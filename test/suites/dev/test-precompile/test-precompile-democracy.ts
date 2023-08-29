import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, ZERO_ADDRESS } from "@moonwall/util";
import { notePreimagePrecompile } from "../../../helpers/precompiles.js";

describeSuite({
  id: "D2530",
  title: "Democracy - genesis and preimage",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should check initial state - no referendum",
      test: async function () {
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        const blockNum = (await context.polkadotJs().rpc.chain.getHeader()).number.toBigInt();
        if (blockNum == 0n) {
          expect(referendumCount.toNumber()).to.equal(0);
        } else {
          log(`Skipping test T01 because block number is ${blockNum}`);
        }
      },
    });

    it({
      id: "T02",
      title: "should check initial state - 0x0 ParachainBondAccount",
      test: async function () {
        const parachainBondInfo = await context
          .polkadotJs()
          .query.parachainStaking.parachainBondInfo();
        expect(parachainBondInfo.account.toString()).to.equal(ZERO_ADDRESS);
      },
    });

    it({
      id: "T03",
      title: "notePreimage",
      test: async function () {
        const encodedHash = await notePreimagePrecompile(
          context,
          context
            .polkadotJs()
            .tx.system.remark(
              "This is a democracy motion for a Revised Grant Program Proposal. " +
                "The full details of the proposal can be found at " +
                "https://github.com/moonbeam-foundation/grants/blob/" +
                "c3cd29629059c61ed8a4c5e9ecd253d5554ff940/revised/revised_grant_program.md"
            )
        );

        const preimageStatus = await context.polkadotJs().query.preimage.statusFor(encodedHash);

        expect(
          preimageStatus.unwrap().isUnrequested &&
            preimageStatus.unwrap().asUnrequested.deposit[0].toString()
        ).to.equal(ALITH_ADDRESS);
        expect(
          preimageStatus.unwrap().isUnrequested &&
            preimageStatus.unwrap().asUnrequested.deposit[1].toString()
        ).to.equal("5024200000000000000");
      },
    });
  },
});
