import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  beforeEach,
  notePreimage,
  execCouncilProposal,
} from "@moonwall/cli";
import { alith } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D010903",
  title: "Democracy - proposing a vote",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let encodedHash: string;
    let randomAddress: string;

    beforeEach(async () => {
      randomAddress = privateKeyToAccount(generatePrivateKey()).address;
      const proposal = context
        .polkadotJs()
        .tx.parachainStaking.setParachainBondAccount(randomAddress);
      encodedHash = await notePreimage(context, proposal, alith);
      await execCouncilProposal(
        context,
        context.polkadotJs().tx.democracy.externalProposeMajority({
          Lookup: {
            hash: encodedHash,
            len: proposal.method.encodedLength,
          },
        })
      );
    });

    it({
      id: "T01",
      title: "should not create a referendum",
      test: async function () {
        const referendumCount = await context.polkadotJs().query.democracy.referendumCount();
        expect(referendumCount.toBigInt()).to.equal(0n);
      },
    });

    it({
      id: "T02",
      title: "should create an external proposal",
      test: async function () {
        const publicProps = await context.polkadotJs().query.democracy.nextExternal();
        expect(publicProps.unwrap()[0].asLookup.hash_.toHex().toString()).to.equal(encodedHash);
      },
    });
  },
});
