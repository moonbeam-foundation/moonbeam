import "@moonbeam-network/api-augment";
import "@polkadot/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { extractWeight } from "@moonwall/util";
import { ApiPromise } from "@polkadot/api";
import { before } from "node:test";
import { blake2AsU8a, xxhashAsU8a } from "@polkadot/util-crypto";
import { hexToU8a, stringToU8a, u8aConcat, u8aToHex } from "@polkadot/util";
import { FrameSystemAccountInfo, PalletBalancesAccountData } from "@polkadot/types/lookup";
import { AccountInfo } from "@polkadot/types/interfaces";
import { GenericExtrinsic, StorageKey, u8, Vec } from "@polkadot/types";
import { AnyTuple } from "@polkadot/types-codec/types";

describeSuite({
  id: "S01",
  title: "Verify random accounts consistency",
  foundationMethods: "read_only",
  testCases: ({ context, it, log }) => {
    beforeAll(async () => {});

    it({
      id: "T01",
      title: "Should verify random accounts consistency",
      test: async function () {
        // Initialize the API
        const api = await context.polkadotJs();

        // Initialize the PRNG with current timestamp
        // To initialize with a fixed seed, use:
        // const seed = 123456;
        // And comment the line below
        const seed = Math.floor(Date.now() / 1000);
        log(`Initializing PRNG with seed: ${seed}`);
        const prng = new SeededPRNG(seed);

        // Pick a random block

        const currentBlockNumber = (await api.rpc.chain.getHeader()).number.toNumber();
        const randomBlockNumber = prng.randomInRange(
          currentBlockNumber - 1000000,
          currentBlockNumber
        );
        console.log(`Fetching block number: ${randomBlockNumber}`);
        const blockHash = await api.rpc.chain.getBlockHash(randomBlockNumber);
        // const blockHash = "0xb5cb4ae95aab87f19e521b5f787cadbc00e053e2beec4ed3fa7982d510122cb9";
        const block = await api.rpc.chain.getBlock(blockHash);

        // Pick a random extrinsic
        let extrinsics;
        let signer = "0x0000000000000000000000000000000000000000";
        let tryCount = 0;

        // If no extrinsics found, keep picking random blocks until we find one with extrinsics
        while (signer == "0x0000000000000000000000000000000000000000") {
          tryCount++;
          log(`Try count: ${tryCount}`);
          const randomBlockNumber = prng.randomInRange(
            currentBlockNumber - 1000000,
            currentBlockNumber
          );
          const blockHash = await api.rpc.chain.getBlockHash(randomBlockNumber);
          const events = await api.query.system.events.at(blockHash);
          const filteredEvents = events.filter(({ event }) => {
            return event.section === "ethereum" && event.method === "Executed";
          });
          if (filteredEvents.length === 0) {
            log(`No ethereum.Executed events found in block ${randomBlockNumber}`);
            tryCount++;
            continue;
          }
          const randomEventIndex = prng.randomInRange(0, filteredEvents.length);
          const randomEvent = filteredEvents[randomEventIndex];
          // Get the signer of the transaction from the event data
          signer = randomEvent.event.data[0].toString();
        }

        console.log(`Selected transaction signer: ${signer}`);

        // Fetch 100 accounts starting from the picked signer
        const startKey = api.query.system.account.key(signer);
        const accounts = await api.query.system.account.entriesPaged({
          args: [],
          pageSize: 100,
          startKey: startKey,
        });
        console.log(`Fetched ${accounts.length} accounts starting from ${signer}:`);
        accounts.forEach(([key, account]) => {
          console.log(`Account: ${key.toString()}, Data: ${account.toString()}`);
        });

        await api.disconnect();
      },
    });
  },
});

class SeededPRNG {
  private seed: number;
  private modulus: number;
  private multiplier: number;
  private increment: number;

  constructor(seed: number) {
    this.seed = seed;
    this.modulus = 2 ** 31 - 1; // A large prime number
    this.multiplier = 48271; // A commonly used multiplier
    this.increment = 0; // Increment is often set to 0 in LCG
  }

  // Generate the next random number
  private next(): number {
    this.seed = (this.multiplier * this.seed + this.increment) % this.modulus;
    return this.seed;
  }

  // Get a random number within a range [min, max)
  public randomInRange(min: number, max: number): number {
    const randomValue = this.next() / this.modulus;
    return Math.floor(randomValue * (max - min) + min);
  }
}
