import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { fromBytes } from "viem";

const SIMPLE_SALT = fromBytes(
  new Uint8Array([..."my_salt".padEnd(32, " ")].map((a) => a.charCodeAt(0))),
  "hex"
);

describeSuite({
  id: "D2706",
  title: "Randomness Babe - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should succeed for 100 random words",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestRelayBabeEpochRandomWords",
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            100, // number of random words
          ],
          gas: 100_000n,
        });
        await context.createBlock([], { signer: alith, allowFailures: false });
        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(1);
      },
    });
  },
});

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   // TODO: This is a flaky and need to be fixed
//   it.skip("should refuse a request with more than 100 random number", test:async function () {
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           101, // number of random words
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
// });
// });

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   // TODO: Flakey test- This intermittently Fails.
//   it.skip("should be marked as pending before the end of the 2nd epoch", test:async function () {
//     const randomnessContract = new context.web3.eth.Contract(
//       RANDOMNESS_CONTRACT_JSON.contract.abi,
//       PRECOMPILE_RANDOMNESS_ADDRESS
//     );

//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           1, // number of random words
//         ]),
//       })
//     );

//     for (let i = 0; i < 10; i++) {
//       await context.createBlock();
//     }

//     expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
//       CONTRACT_RANDOMNESS_STATUS_PENDING.toString()
//     );
//   });
// });

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   // TODO: Fix it once we support setting the epochs properly
//   it.skip("should be marked as ready after 2 epochs has passed", test:async function () {
//     const randomnessContract = new context.web3.eth.Contract(
//       RANDOMNESS_CONTRACT_JSON.contract.abi,
//       PRECOMPILE_RANDOMNESS_ADDRESS
//     );

//     await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           1, // number of random words
//         ]),
//       })
//     );

//     expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
//       CONTRACT_RANDOMNESS_STATUS_READY.toString()
//     );
//   });
// });
