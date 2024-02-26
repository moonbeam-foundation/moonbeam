import "@moonbeam-network/api-augment/moonbase";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013118",
  title: "Randomness VRF - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "requestLocalVRFRandomWords",
        args: [alith.address, 1n * GLMR, 100_000n, SIMPLE_SALT, 1, 2],
        gas: 100_000n,
      });
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should store a request with id:0",
      test: async function () {
        const requestId = parseInt(
          ((await context.polkadotJs().query.randomness.requests.entries()) as any)[0][0]
            .toHex()
            .slice(-16),
          16
        );
        expect(requestId).to.equal(0);
      },
    });

    it({
      id: "T02",
      title: "should store the salt",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.salt.toHex()).to.equal(SIMPLE_SALT);
      },
    });

    it({
      id: "T03",
      title: "should store the refundAddress",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.refundAddress.toHex()).to.equal(alith.address.toLocaleLowerCase());
      },
    });

    // This is a bit weird as we are calling the precompile from a non smart-contract
    it({
      id: "T04",
      title: "should store the contractAddress",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.contractAddress.toHex()).to.equal(alith.address.toLocaleLowerCase());
      },
    });

    it({
      id: "T05",
      title: "should store the fee",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.fee.toBigInt()).to.equal(1n * GLMR);
      },
    });

    it({
      id: "T06",
      title: "should store the gasLimit",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.gasLimit.toBigInt()).to.equal(100_000n);
      },
    });

    it({
      id: "T07",
      title: "should store the numWords",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.numWords.toBigInt()).to.equal(1n);
      },
    });

    it({
      id: "T08",
      title: "should be considered a local vrf type",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.info.isLocal).to.be.true;
      },
    });

    it({
      id: "T09",
      title: "should have a fulfillment block of 3",
      test: async function () {
        const request = (
          (await context.polkadotJs().query.randomness.requests.entries()) as any
        )[0][1].unwrap().request;
        expect(request.info.asLocal[0].toBigInt()).to.be.equal(3n);
      },
    });

    it({
      id: "T10",
      title: "should have an expiration block of 10001",
      test: async function () {
        const request = (
          (await context.polkadotJs().query.randomness.requests.entries()) as any
        )[0][1].unwrap().request;
        expect(request.info.asLocal[1].toBigInt()).to.be.equal(10001n);
      },
    });
  },
});
