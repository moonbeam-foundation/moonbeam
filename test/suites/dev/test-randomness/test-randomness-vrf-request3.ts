import "@moonbeam-network/api-augment/moonbase";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { expectEVMResult, extractRevertReason } from "../../../helpers/eth-transactions.js";
import { SIMPLE_SALT } from "../../../helpers/randomness.js";

describeSuite({
  id: "D2719",
  title: "Randomness VRF - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should refuse a request with less than 2 blocks",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, 1],
          gas: 100_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;
        expectEVMResult(result!.events, "Revert");

        const revertReason = await extractRevertReason(result!.hash, context.ethers() as any);
        // Full error expected:
        // Error in pallet_randomness: Module(ModuleError { index: 39, error: [5, 0, 0, 0],
        // message: Some("CannotRequestRandomnessBeforeMinDelay") })
        expect(revertReason).to.contain("CannotRequestRandomnessBeforeMinDelay");

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "should refuse a request with more than 2000 blocks",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestLocalVRFRandomWords",
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
            2001, // future blocks
          ],
          gas: 100_000n,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);

        expect(result!.successful).to.be.true;
        expectEVMResult(result!.events, "Revert");

        const revertReason = await extractRevertReason(result!.hash, context.ethers() as any);
        // Full error expected:
        // Error in pallet_randomness: Module(ModuleError { index: 39, error: [4, 0, 0, 0],
        // message: Some("CannotRequestRandomnessAfterMaxDelay") })
        expect(revertReason).to.contain("CannotRequestRandomnessAfterMaxDelay");

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(0);
      },
    });
  },
});

// describeSuite({id:"", title:"Randomness VRF - Requesting a random number",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   it({id:"",title:"should refuse a request with less than 1 random number",test: async function () {
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           0, // number of random words
//           2, // future blocks
//         ]),
//       })
//     );

//     expect(result.successful).to.be.true;
//     expectEVMResult(result.events, "Revert");
//     const revertReason = await extractRevertReason(result.hash, context.ethers);
//     // Full error expected:
//     // Error in pallet_randomness: Module(ModuleError { index: 39, error: [2, 0, 0, 0],
//     // message: Some("MustRequestAtLeastOneWord") })
//     expect(revertReason).to.contain("MustRequestAtLeastOneWord");

//     const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//     expect(randomnessRequests.length).to.equal(0);
//   });
// });

// describeSuite({id:"", title:"Randomness VRF - Requesting a random number",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   it({id:"",title:"should refuse a request with more than 100 random number",test: async function () {
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           101, // number of random words
//           2, // future blocks
//         ]),
//       })
//     );

//     expect(result.successful).to.be.true;
//     expectEVMResult(result.events, "Revert");
//     const revertReason = await extractRevertReason(result.hash, context.ethers);
//     // Full error expected:
//     // Error in pallet_randomness: Module(ModuleError { index: 39, error: [3, 0, 0, 0],
//     // message: Some("CannotRequestMoreWordsThanMax") })
//     expect(revertReason).to.contain("CannotRequestMoreWordsThanMax");

//     const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//     expect(randomnessRequests.length).to.equal(0);
//   });
// });

// describeSuite({id:"", title:"Randomness VRF - Requesting a random number",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   it({id:"",title:"should succeed for 100 random words",test: async function () {
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           100, // number of random words
//           2, // future blocks
//         ]),
//       })
//     );

//     expect(result.successful).to.be.true;

//     const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//     expect(randomnessRequests.length).to.equal(1);
//   });
// });

// describeSuite({id:"", title:"Randomness VRF - Requesting a random number",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   it({id:"",title:"should be marked as pending before the end of the delay",test: async function () {
//     const randomnessContract = new context.web3.eth.Contract(
//       RANDOMNESS_CONTRACT_JSON.contract.abi,
//       PRECOMPILE_RANDOMNESS_ADDRESS
//     );

//     const delayBlocks = 4;

//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           1, // number of random words
//           delayBlocks, // future blocks
//         ]),
//       })
//     );

//     for (let i = 0; i < delayBlocks - 1; i++) {
//       await context.createBlock();
//     }

//     expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
//       CONTRACT_RANDOMNESS_STATUS_PENDING.toString()
//     );
//   });
// });

// describeSuite({id:"", title:"Randomness VRF - Requesting a random number",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   it({id:"",title:"should be marked as ready after delay has passed",test: async function () {
//     const randomnessContract = new context.web3.eth.Contract(
//       RANDOMNESS_CONTRACT_JSON.contract.abi,
//       PRECOMPILE_RANDOMNESS_ADDRESS
//     );

//     const delayBlocks = 3;

//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           1, // number of random words
//           delayBlocks, // future blocks
//         ]),
//       })
//     );

//     for (let i = 0; i < delayBlocks; i++) {
//       await context.createBlock();
//     }

//     expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
//       CONTRACT_RANDOMNESS_STATUS_READY.toString()
//     );
//   });
// });

// describeSuite({id:"", title:"Randomness VRF - Requesting an invalid random number",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   it({id:"",title:"should be marked as does not exists",test: async function () {
//     const randomnessContract = new context.web3.eth.Contract(
//       RANDOMNESS_CONTRACT_JSON.contract.abi,
//       PRECOMPILE_RANDOMNESS_ADDRESS
//     );

//     expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
//       CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS.toString()
//     );
//   });
// });

// describeSuite({id:"", title:"Randomness VRF - Fulfilling a random request",foundationMethods:"dev",testCases: ({context, it, log}) => {
//   before("setup the request",test: async function () {
//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           1, // number of random words
//           2, // future blocks
//         ]),
//       })
//     );
//     await context.createBlock();
//     await context.createBlock();

//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
//       })
//     );
//   });

//   it({id:"",title:"should remove the request",test: async function () {
//     const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//     expect(randomnessRequests.length).to.equal(0);
//   });

//   it({id:"",title:"should remove the randomness results",test: async function () {
//     const randomnessResults =
//       await context.polkadotJs().query.randomness.randomnessResults.entries();
//     expect(randomnessResults.length).to.equal(0);
//   });
// });

// describeDevMoonbeam(
//   "Randomness VRF - Requesting 2 random requests at same block/delay",
//   (context) => {
//     before("setup the request",test: async function () {
//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           to: PRECOMPILE_RANDOMNESS_ADDRESS,
//           data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//             alith.address, // refund address
//             1n * GLMR, // fee
//             100_000n, // gas limit
//             SIMPLE_SALT,
//             1, // number of random words
//             3, // future blocks
//           ]),
//         })
//       );
//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           to: PRECOMPILE_RANDOMNESS_ADDRESS,
//           data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//             alith.address, // refund address
//             1n * GLMR, // fee
//             100_000n, // gas limit
//             SIMPLE_SALT,
//             1, // number of random words
//             2, // future blocks
//           ]),
//         })
//       );
//       // printEvents(context.polkadotJs());
//     });

//     it({id:"",title:"should create 2 requests",test: async function () {
//       const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//       // console.log(randomnessRequests);
//       expect(randomnessRequests.length).to.equal(2);
//     });

//     it({id:"",title:"should have 1 random result",test: async function () {
//       const randomnessResults =
//         await context.polkadotJs().query.randomness.randomnessResults.entries();
//       expect(randomnessResults.length).to.equal(1);
//     });
//   }
// );

// describeSuite({id:"", title:
//   "Randomness VRF - Fulfilling one of the 2 random requests at same block/delay", foundationMethods:"dev", testCases:
//   ({context, it,log}) => {
//     beforeAll( async function () {
//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           to: PRECOMPILE_RANDOMNESS_ADDRESS,
//           data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//             alith.address, // refund address
//             1n * GLMR, // fee
//             100_000n, // gas limit
//             SIMPLE_SALT,
//             1, // number of random words
//             3, // future blocks
//           ]),
//         })
//       );
//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           to: PRECOMPILE_RANDOMNESS_ADDRESS,
//           data: RANDOMNESS_INTERFACE.encodeFunctionData("requestLocalVRFRandomWords", [
//             alith.address, // refund address
//             1n * GLMR, // fee
//             100_000n, // gas limit
//             SIMPLE_SALT,
//             1, // number of random words
//             2, // future blocks
//           ]),
//         })
//       );
//       await context.createBlock();
//       await context.createBlock();

//       await context.createBlock(
//         createTransaction(context, {
//           ...ALITH_TRANSACTION_TEMPLATE,
//           to: PRECOMPILE_RANDOMNESS_ADDRESS,
//           data: RANDOMNESS_INTERFACE.encodeFunctionData("fulfillRequest", [0]),
//         })
//       );
//     });

//     it({id:"",title:"should keep the 2nd request",test: async function () {
//       const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//       expect(randomnessRequests.length).to.equal(1);
//     }});

//     it({id:"",title:"should keep the randomness results",test: async function () {
//       const randomnessResults =
//         await context.polkadotJs().query.randomness.randomnessResults.entries();
//       expect(randomnessResults.length).to.equal(1);
//     }});
//   }
// });
