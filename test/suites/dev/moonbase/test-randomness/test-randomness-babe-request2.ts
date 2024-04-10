import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { GLMR, alith } from "@moonwall/util";
import { SIMPLE_SALT } from "../../../../helpers";

describeSuite({
  id: "D013105",
  title: "Randomness Babe - Requesting a random number",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    beforeAll(async function () {
      await context.writePrecompile!({
        precompileName: "Randomness",
        functionName: "requestRelayBabeEpochRandomWords",
        args: [
          alith.address, // refund address
          1n * GLMR, // fee
          120_000n, // gas limit
          SIMPLE_SALT,
          1, // number of random words
        ],
        gas: 120_000n,
      });

      await context.createBlock([], { signer: alith, allowFailures: false });
    });

    it({
      id: "T01",
      title: "should store a request with id:0",
      test: async function () {
        const requestId = parseInt(
          (await context.polkadotJs().query.randomness.requests.entries())[0][0].toHex().slice(-16),
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
        expect(request.gasLimit.toBigInt()).to.equal(120_000n);
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
      title: "should be considered a babe type",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.info.isBabeEpoch).to.be.true;
      },
    });

    it({
      id: "T09",
      title: "should have a fulfillment delay of 2 epochs",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.info.asBabeEpoch[0].toBigInt()).to.be.equal(2n);
      },
    });

    it({
      id: "T10",
      title: "should have an expiration delay of 10001 epochs",
      test: async function () {
        const request = (
          await context.polkadotJs().query.randomness.requests.entries()
        )[0][1].unwrap().request;
        expect(request.info.asBabeEpoch[1].toBigInt()).to.be.equal(10000n);
      },
    });
  },
});
