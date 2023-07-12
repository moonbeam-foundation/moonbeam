import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  DOROTHY_ADDRESS
} from "@moonwall/util";
import { setupPoolWithParticipants } from "../../../helpers/precompiles.js";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";

describeSuite({
  id: "D2544",
  title: "Proxy Leader Demo - Vote",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let leaderContractAddress: `0x${string}`;

    beforeAll(async function () {
      leaderContractAddress = await setupPoolWithParticipants(context);
      const rawTx = context.writeContract!({
        contractName: "ProxyLeaderDemo",
        contractAddress: leaderContractAddress,
        functionName: "startVoting",
        rawTxOnly: true,
      });
      const { result } = await context.createBlock(rawTx);
      expectEVMResult(result!.events, "Succeed");
    });

    it({
      id: "T01",
      title: "should not be able to vote if non-participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [ALITH_ADDRESS],
          })
        ).to.be.false;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [CHARLETH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          gas: 1000000n,
        });
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Revert");
      },
    });

    it({
      id: "T02",
      title: "should not be able to vote for non-participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [ALITH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 1000000n,
        });
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Revert");
      },
    });

    it({
      id: "T03",
      title: "should be able to vote for participant when participant",
      test: async function () {
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.true;

        const rawTx = context.writeContract!({
          contractName: "ProxyLeaderDemo",
          contractAddress: leaderContractAddress,
          functionName: "vote",
          args: [CHARLETH_ADDRESS, DOROTHY_ADDRESS],
          rawTxOnly: true,
          privateKey: BALTATHAR_PRIVATE_KEY,
          gas: 1_000_000n,
        });
        const { result } = await context.createBlock(rawTx);

        expectEVMResult(result!.events, "Succeed");
        expect(
          await context.readContract!({
            contractAddress: leaderContractAddress,
            contractName: "ProxyLeaderDemo",
            functionName: "canVote",
            args: [BALTATHAR_ADDRESS],
          })
        ).to.be.false;
      },
    });
  },
});

// describeDevMoonbeam("Proxy Leader Demo - End Voting", (context) => {
//   let leaderContract: Contract;
//   before("setup contract and start voting", test: async function () {
//     leaderContract = await setupPoolWithParticipants(context);

//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("startVoting", []),
//       })
//     );
//     expectEVMResult(result.events, "Succeed");
//   });

//   // TODO: rework this test, contract cannot call proxy precompile
//   it.skip("should be able to stop", test: async function () {
//     expect(await leaderContract.methods.isVoting().call()).to.be.true;

//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("endVoting", []),
//       })
//     );
//     expectEVMResult(result.events, "Succeed");

//     expect(await leaderContract.methods.isVoting().call()).to.be.false;
//  } });
// });

// // TODO: rework this test, contract cannot call proxy precompile
// describeDevMoonbeam("Proxy Leader Demo - Winners", (context) => {
//   let leaderContract: Contract;

//   before("setup contract and voting results", test: async function () {
//     this.skip();
//     leaderContract = await setupPoolWithParticipants(context);

//     // start voting
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("startVoting", []),
//       })
//     );
//     expectEVMResult(result.events, "Succeed");

//     // baltathar votes
//     const { result: resultVote1 } = await context.createBlock(
//       createTransaction(context, {
//         ...BALTATHAR_TRANSACTION_TEMPLATE,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("vote", [CHARLETH_ADDRESS, DOROTHY_ADDRESS]),
//       })
//     );
//     expectEVMResult(resultVote1.events, "Succeed");

//     // charleth votes
//     const { result: resultVote2 } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         from: CHARLETH_ADDRESS,
//         privateKey: CHARLETH_PRIVATE_KEY,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("vote", [BALTATHAR_ADDRESS, BALTATHAR_ADDRESS]),
//       })
//     );
//     expectEVMResult(resultVote2.events, "Succeed");

//     // dorothy votes
//     const { result: resultVote3 } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         from: DOROTHY_ADDRESS,
//         privateKey: DOROTHY_PRIVATE_KEY,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("vote", [CHARLETH_ADDRESS, DOROTHY_ADDRESS]),
//       })
//     );
//     expectEVMResult(resultVote3.events, "Succeed");

//     // end voting
//     const { result: resultEnd } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: leaderContract.options.address,
//         data: LEADER_INTERFACE.encodeFunctionData("endVoting", []),
//       })
//     );
//     expectEVMResult(resultEnd.events, "Succeed");

//     // create a referendum
//     await instantFastTrack(context, context.polkadotJs().tx.system.remark("foobar"), {
//       votingPeriod: 10,
//       delayPeriod: 0,
//     });
//   });

//   it.skip("should proxy charleth as governor", test: async function () {
//     expect(await leaderContract.methods.governor().call()).to.equal(CHARLETH_ADDRESS);
//   });

//   it.skip("should proxy dorothy as staker", test: async function () {
//     expect(await leaderContract.methods.staker().call()).to.equal(DOROTHY_ADDRESS);
//   });

//   it.skip("should setup proxy types for contract address", test: async function () {
//     const proxies = await context.polkadotJs().query.proxy.proxies(leaderContract.options.address);
//     expect(proxies[0].toJSON()).to.deep.equal([
//       {
//         delegate: DOROTHY_ADDRESS,
//         proxyType: "Staking",
//         delay: 0,
//       },
//       {
//         delegate: CHARLETH_ADDRESS,
//         proxyType: "Governance",
//         delay: 0,
//       },
//     ]);
//   });

//   it.skip("should not allow baltathar to stake via proxy", test: async function () {
//     const { result } = await context.createBlock(
//       context.polkadotJs().tx.proxy
//         .proxy(
//           leaderContract.options.address,
//           "Staking",
//           context.polkadotJs().tx.parachainStaking.delegate(ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 0, 0)
//         )
//         .signAsync(baltathar)
//     );
//     expect(result.successful).to.be.false;
//     expect(result.error.name).to.equal("NotProxy");
//   });

//   it.skip("should allow dorothy to stake via proxy", test: async function () {
//     const { result } = await context.createBlock(
//       context.polkadotJs().tx.proxy
//         .proxy(
//           leaderContract.options.address,
//           "Staking",
//           context.polkadotJs().tx.parachainStaking.delegate(ALITH_ADDRESS, MIN_GLMR_DELEGATOR, 0, 0)
//         )
//         .signAsync(dorothy)
//     );

//     const delegationEvents = result.events.reduce((acc, event) => {
//       if (context.polkadotJs().events.parachainStaking.Delegation.is(event.event)) {
//         acc.push({
//           delegator: event.event.data[0].toString(),
//           candidate: event.event.data[2].toString(),
//         });
//       }
//       return acc;
//     }, []);

//     expect(result.successful).to.be.true;
//     expect(delegationEvents).to.deep.equal([
//       {
//         delegator: leaderContract.options.address,
//         candidate: ALITH_ADDRESS,
//       },
//     ]);
//   });

//   it.skip("should not allow dorothy to vote via proxy", test: async function () {
//     const { result } = await context.createBlock(
//       context.polkadotJs().tx.proxy
//         .proxy(
//           leaderContract.options.address,
//           "Governance",
//           context.polkadotJs().tx.democracy.vote(0, {
//             Standard: { balance: 10n * GLMR, vote: { aye: true, conviction: 1 } },
//           })
//         )
//         .signAsync(dorothy)
//     );
//     expect(result.successful).to.be.false;
//     expect(result.error.name).to.equal("NotProxy");
//   });

//   it.skip("should allow charleth to vote via proxy", test: async function () {
//     const { result } = await context.createBlock(
//       context.polkadotJs().tx.proxy
//         .proxy(
//           leaderContract.options.address,
//           "Governance",
//           context.polkadotJs().tx.democracy.vote(0, {
//             Standard: { balance: 1n * GLMR, vote: { aye: true, conviction: 1 } },
//           })
//         )
//         .signAsync(charleth)
//     );

//     const votedEvents = result.events.reduce((acc, event) => {
//       if (context.polkadotJs().events.democracy.Voted.is(event.event)) {
//         acc.push({
//           voter: event.event.data[0].toString(),
//           isAye: event.event.data[2].asStandard.vote.isAye,
//         });
//       }
//       return acc;
//     }, []);

//     expect(result.successful).to.be.true;
//     expect(votedEvents).to.deep.equal([
//       {
//         voter: leaderContract.options.address,
//         isAye: true,
//       },
//     ]);
//   });
// });
