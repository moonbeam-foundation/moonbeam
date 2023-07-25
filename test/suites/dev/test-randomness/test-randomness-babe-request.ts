import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { fromBytes } from "viem";
// import { u8aToHex } from "@polkadot/util";
// import { expect } from "chai";
// import { ethers } from "ethers";
// import { alith } from "../../util/accounts";
// import {
//   CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS,
//   CONTRACT_RANDOMNESS_STATUS_PENDING,
//   CONTRACT_RANDOMNESS_STATUS_READY,
//   GLMR,
//   PRECOMPILE_RANDOMNESS_ADDRESS,
// } from "../../util/constants";
// import { getCompiled } from "../../util/contracts";
// import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";
// import { describeSuite } from "../../util/setup-dev-tests";
// import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";

// const RANDOMNESS_CONTRACT_JSON = getCompiled("precompiles/randomness/Randomness");
// const RANDOMNESS_INTERFACE = new ethers.utils.Interface(RANDOMNESS_CONTRACT_JSON.contract.abi);

const SIMPLE_SALT = fromBytes(
  new Uint8Array([..."my_salt".padEnd(32, " ")].map((a) => a.charCodeAt(0))),
  "hex"
);

describeSuite({
  id: "D2704",
  title: "Randomness Babe - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should be successful",
      test: async function () {
        await context.writePrecompile!({
          precompileName: "Randomness",
          functionName: "requestRelayBabeEpochRandomWords",
          gas: 100_000n,
          args: [
            alith.address, // refund address
            1n * GLMR, // fee
            100_000n, // gas limit
            SIMPLE_SALT,
            1, // number of random words
          ],
        });
        await context.createBlock([], { signer: alith, allowFailures: false });

        const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
        expect(randomnessRequests.length).to.equal(1);
        log(
          `Randomness returned for ${randomnessRequests[0][1]
            .unwrap()
            .request.numWords.toNumber()} words`
        );
      },
    });
  },
});

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   before("setup the request", async function () {
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
//   });

//   it("should store a request with id:0", async function () {
//     const requestId = parseInt(
//       ((await context.polkadotJs().query.randomness.requests.entries()) as any)[0][0]
//         .toHex()
//         .slice(-16),
//       16
//     );
//     expect(requestId).to.equal(0);
//   });

//   it("should store the salt", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.salt.toHex()).to.equal(u8aToHex(SIMPLE_SALT));
//   });

//   it("should store the refundAddress", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.refundAddress.toHex()).to.equal(alith.address.toLocaleLowerCase());
//   });

//   // This is a bit weird as we are calling the precompile from a non smart-contract
//   it("should store the contractAddress", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.contractAddress.toHex()).to.equal(alith.address.toLocaleLowerCase());
//   });

//   it("should store the fee", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.fee.toBigInt()).to.equal(1n * GLMR);
//   });

//   it("should store the gasLimit", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.gasLimit.toBigInt()).to.equal(100_000n);
//   });

//   it("should store the numWords", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.numWords.toBigInt()).to.equal(1n);
//   });

//   it("should be considered a babe type", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.info.isBabeEpoch).to.be.true;
//   });

//   it("should have a fulfillment delay of 2 epochs", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.info.asBabeEpoch[0].toBigInt()).to.be.equal(2n);
//   });

//   it("should have an expiration delay of 10001 epochs", async function () {
//     const request = (
//       (await context.polkadotJs().query.randomness.requests.entries()) as any
//     )[0][1].unwrap().request;
//     expect(request.info.asBabeEpoch[1].toBigInt()).to.be.equal(10000n);
//   });
// });

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   // TODO: This is a flaky and need to be fixed
//   it.skip("should refuse a request with more than 100 random number", async function () {
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

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   it("should succeed for 100 random words", async function () {
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_RANDOMNESS_ADDRESS,
//         data: RANDOMNESS_INTERFACE.encodeFunctionData("requestRelayBabeEpochRandomWords", [
//           alith.address, // refund address
//           1n * GLMR, // fee
//           100_000n, // gas limit
//           SIMPLE_SALT,
//           100, // number of random words
//         ]),
//       })
//     );

//     expect(result.successful).to.be.true;

//     const randomnessRequests = await context.polkadotJs().query.randomness.requests.entries();
//     expect(randomnessRequests.length).to.equal(1);
//   });
// });

// describeSuite({id:"",title:"Randomness Babe - Requesting a random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   // TODO: Flakey test- This intermittently Fails.
//   it.skip("should be marked as pending before the end of the 2nd epoch", async function () {
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
//   it.skip("should be marked as ready after 2 epochs has passed", async function () {
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

// describeSuite({id:"",title:"Randomness Babe - Requesting an invalid random number", foundationMethods:"dev",testCases:({context,it,log}) => {
//   it("should be marked as pending before the end of the delay", async function () {
//     const randomnessContract = new context.web3.eth.Contract(
//       RANDOMNESS_CONTRACT_JSON.contract.abi,
//       PRECOMPILE_RANDOMNESS_ADDRESS
//     );

//     expect(await randomnessContract.methods.getRequestStatus(0).call()).to.equal(
//       CONTRACT_RANDOMNESS_STATUS_DOES_NOT_EXISTS.toString()
//     );
//   });
// });
