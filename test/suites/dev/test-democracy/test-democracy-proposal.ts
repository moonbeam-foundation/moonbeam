import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeEach, notePreimage } from "@moonwall/cli";
import { GLMR, PROPOSAL_AMOUNT, alith } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";

describeSuite({
  id: "D0806",
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
      await context.createBlock(
        context.polkadotJs().tx.democracy.propose(
          {
            Lookup: {
              hash: encodedHash,
              len: proposal.method.encodedLength,
            },
          },
          PROPOSAL_AMOUNT
        )
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
      title: "should increase the number of proposals to 2",
      test: async function () {
        const publicPropCount = await context.polkadotJs().query.democracy.publicPropCount();
        expect(publicPropCount.toBigInt()).to.equal(2n);
      },
    });

    it({
      id: "T03",
      title: "should create a proposal",
      test: async function () {
        const publicProps = await context.polkadotJs().query.democracy.publicProps();

        expect(publicProps[publicProps.length - 1][1].asLookup.hash_.toHex().toString()).to.equal(
          encodedHash
        );
        expect(publicProps[publicProps.length - 1][2].toString()).toBe(alith.address);
      },
    });

    it({
      id: "T04",
      title: "should include a deposit of 1000 TOKENs",
      test: async function () {
        const depositOf = await context.polkadotJs().query.democracy.depositOf(0);
        expect(depositOf.unwrap()[1].toBigInt()).to.equal(1000n * GLMR);
      },
    });
  },
});

// describeDevMoonbeam("Democracy - Seconding a proposal", (context) => {
//   let encodedHash: string;
//   let launchPeriod: u32;

//   before("Setup genesis account for substrate", async () => {
//     const proposal = context.polkadotJs().tx.parachainStaking.setParachainBondAccount(alith.address);
//     const encodedProposal = proposal.method.toHex() || "";

//     //launchPeriod
//     launchPeriod = await context.polkadotJs().consts.democracy.launchPeriod;

//     // notePreimage
//     encodedHash = await notePreimage(context, proposal, alith);

//     // propose & second
//     await context.createBlock(
//       context.polkadotJs().tx.democracy.propose(
//         {
//           Lookup: {
//             hash: encodedHash,
//             len: proposal.method.encodedLength,
//           },
//         } as any,
//         PROPOSAL_AMOUNT
//       )
//     );
//     await context.createBlock((context.polkadotJs().tx.democracy as any).second(0));
//   }});

//   it({id:"T0",title:"should succeed", test: async function () {
//     // publicProps
//     // TODO: Remove any casting when api-augment is updated
//     const publicProps = (await context.polkadotJs().query.democracy.publicProps()) as any;
//     // encodedHash
//     expect(publicProps[0][1].asLookup.hash_.toHex().toString()).to.equal(encodedHash);
//     // prop author
//     expect(publicProps[0][2].toString()).to.equal(alith.address);

//     // depositOf
//     const depositOf = await context.polkadotJs().query.democracy.depositOf(0);
//     expect(depositOf.unwrap()[1].toBigInt()).to.equal(1000n * GLMR);
//     expect(depositOf.unwrap()[0][1].toString()).to.equal(alith.address);
//   }});

//   it({id:"T0",title:"should have a launch period of 7200", test: async function () {
//     // launchPeriod
//     expect(launchPeriod.toBigInt()).to.equal(7200n);
//   }});
// }});

// describeDevMoonbeam("Democracy - Seconding a proposal", (context) => {
//   let encodedHash: string;
//   let launchPeriod: u32;

//   before("Setup genesis account for substrate", async () => {
//     const proposal = context.polkadotJs().tx.parachainStaking.setParachainBondAccount(alith.address);
//     const encodedProposal = proposal.method.toHex() || "";

//     //launchPeriod
//     launchPeriod = await context.polkadotJs().consts.democracy.launchPeriod;

//     // notePreimage
//     encodedHash = await notePreimage(context, proposal, alith);

//     // propose & second
//     await context.createBlock(
//       context.polkadotJs().tx.democracy.propose(
//         {
//           Lookup: {
//             hash: encodedHash,
//             len: proposal.method.encodedLength,
//           },
//         } as any,
//         PROPOSAL_AMOUNT
//       )
//     );
//     await context.createBlock((context.polkadotJs().tx.democracy as any).second(0));
//   }});

//   it({id:"T0",title:"should end-up in a valid referendum", test: async function () {
//     this.timeout(1000000);
//     // let Launchperiod elapse to turn the proposal into a referendum
//     // launchPeriod minus the 3 blocks that already elapsed
//     for (let i = 0; i < Number(launchPeriod) - 3; i++) {
//       await context.createBlock();
//     }
//     // referendumCount
//     let referendumCount = await context.polkadotJs().query.democracy.referendumCount();
//     expect(referendumCount.toHuman()).to.equal("1");

//     // publicPropCount
//     const publicPropCount = await context.polkadotJs().query.democracy.publicPropCount();
//     expect(publicPropCount.toHuman()).to.equal("1");

//     // referendumInfoOf
//     const referendumInfoOf = await context.polkadotJs().query.democracy.referendumInfoOf(0);
//     // TODO: Remove any casting when api-augment is updated
//     expect((referendumInfoOf.unwrap() as any).asOngoing.proposal.asLookup.hash_.toHex()).to.equal(
//       encodedHash
//     );
//   }});
// }});
