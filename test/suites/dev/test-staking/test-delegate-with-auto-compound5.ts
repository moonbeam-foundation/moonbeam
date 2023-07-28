// import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  MIN_GLMR_DELEGATOR,
  MIN_GLMR_STAKING,
  alith,
  baltathar,
  charleth,
  ethan,
} from "@moonwall/util";

describeSuite({
  id: "D2916",
  title: "Staking - Delegate With Auto-Compound - wrong delegation hint",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    beforeAll(async () => {
      await context.createBlock(
        [
          context
            .polkadotJs()
            .tx.parachainStaking.joinCandidates(MIN_GLMR_STAKING, 1)
            .signAsync(baltathar),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(charleth),
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              baltathar.address,
              MIN_GLMR_DELEGATOR,
              50,
              0,
              0,
              0
            )
            .signAsync(ethan),
        ],
        { signer: alith, allowFailures: false }
      );
    });

    it({
      id: "T01",
      title: "should fail",
      test: async () => {
        const block = await context.createBlock(
          context
            .polkadotJs()
            .tx.parachainStaking.delegateWithAutoCompound(
              alith.address,
              MIN_GLMR_DELEGATOR,
              50,
              1,
              0,
              0
            )
            .signAsync(ethan)
        );
        expect(block.result!.successful).to.be.false;
        expect(block.result!.error!.name).to.equal("TooLowDelegationCountToDelegate");
      },
    });
  },
});

// describeSuite({id:"D2920",title:"Staking - Delegate With Auto-Compound - 101%",foundationMethods:"dev",testCases: ({it,log,context}) => {
//   it({id:"T01",title:"should fail",test: async () => {
//     await expect(
//       context.createBlock(
//         context.polkadotJs().tx.parachainStaking
//           .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 101, 0, 0, 0)
//           .signAsync(ethan)
//       )
//     ).to.eventually.be.rejectedWith("Value is greater than allowed maximum!");
//   } });
// }});

// describeSuite({id:"D2921",title:"Staking - Delegate With Auto-Compound - valid request",foundationMethods:"dev",testCases: ({it,log,context}) => {
//   const numberToHex = (n: BigInt): string => `0x${n.toString(16).padStart(32, "0")}`;

//   let events;
//   beforeAll(should delegate",test: async () => {
//     const { result } = await context.createBlock(
//       context.polkadotJs().tx.parachainStaking
//         .delegateWithAutoCompound(alith.address, MIN_GLMR_DELEGATOR, 50, 0, 0, 0)
//         .signAsync(ethan)
//     );
//     expect(result.successful).to.be.true;
//     events = result.events;
//   });

//   it({id:"T01",title:"should succeed",test: async () => {
//     const delegatorState = await context.polkadotJs().query.parachainStaking.delegatorState(
//       ethan.address
//     );
//     const autoCompoundConfig = (
//       (await context.polkadotJs().query.parachainStaking.autoCompoundingDelegations(
//         alith.address
//       )) as any
//     )
//       .toJSON()
//       .find((d) => d.delegator === ethan.address);
//     const delegationEvents = events.reduce((acc, event) => {
//       if (context.polkadotJs().events.parachainStaking.Delegation.is(event.event)) {
//         acc.push({
//           account: event.event.data[0].toString(),
//           amount: event.event.data[1].toBigInt(),
//           autoCompound: event.event.data[4].toJSON(),
//         });
//       }
//       return acc;
//     }, []);

//     expect(delegationEvents).to.deep.equal([
//       {
//         account: ethan.address,
//         amount: 1000000000000000000n,
//         autoCompound: 50,
//       },
//     ]);
//     expect(delegatorState.unwrap().toJSON()).to.deep.equal({
//       delegations: [
//         {
//           amount: numberToHex(MIN_GLMR_DELEGATOR),
//           owner: alith.address,
//         },
//       ],
//       id: ethan.address,
//       lessTotal: 0,
//       status: { active: null },
//       total: numberToHex(MIN_GLMR_DELEGATOR),
//     });
//     expect(autoCompoundConfig).to.deep.equal({
//       delegator: ethan.address,
//       value: 50,
//     });
//   } });
// }});
