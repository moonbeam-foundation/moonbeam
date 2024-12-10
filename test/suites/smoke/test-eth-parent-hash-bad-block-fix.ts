import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { THIRTY_MINS } from "@moonwall/util";
import type { ApiDecoration } from "@polkadot/api/types";

describeSuite({
  id: "S07",
  title: `RPC Eth ParentHash`,
  foundationMethods: "dev",
  testCases: async function ({ context, it, log }) {
    let atBlockNumber: bigint;
    let previousBlockNumber: bigint;
    let apiAt: ApiDecoration<"promise">;

    beforeAll(async function () {
      atBlockNumber = (await context.polkadotJs("para").rpc.chain.getHeader()).number.toBigInt();
      log(`Testing at block #${atBlockNumber}`);
      previousBlockNumber = atBlockNumber - 1n;
      apiAt = await context
        .polkadotJs("para")
        .at(await context.polkadotJs("para").rpc.chain.getBlockHash(atBlockNumber));
    }, THIRTY_MINS);

    it({
      id: "C100",
      title: "should return correct parent hash via rpc for current block",
      test: async function () {
        const rpcParentHash = (
          await context.viem().getBlock({ blockNumber: atBlockNumber, includeTransactions: false })
        ).parentHash;
        const storedParentHash = (await apiAt.query.ethereum.currentBlock()).unwrap().header
          .parentHash;

        const actualParentHash = (
          await context
            .viem()
            .getBlock({ blockNumber: previousBlockNumber, includeTransactions: false })
        ).hash;
        expect(storedParentHash.isEmpty, "stored parentHash was empty").to.be.false;
        expect(rpcParentHash.toString()).to.equal(actualParentHash.toString());

        log(
          `Verified ethereum parentHash ${rpcParentHash} for block #${atBlockNumber} \
          (at #${atBlockNumber})`
        );
      },
    });

    it({
      id: "C200",
      title: "should return correct parent hash via rpc for block #1648995",
      test: async function () {
        const badBlockNumber = 1648995;
        const apiAtBadBlock = await context
          .polkadotJs("para")
          .at(await context.polkadotJs("para").rpc.chain.getBlockHash(badBlockNumber));

        const chainName = (await context.polkadotJs("para").rpc.system.chain()).toString();
        if (chainName !== "Moonbase Alpha") {
          log(`Test only applies for "Moonbase Alpha", skipping for "${chainName}"`);
          return;
        }

        const rpcParentHash = (
          await context.polkadotJs("para").rpc.eth.getBlockByNumber(badBlockNumber, false)
        ).unwrap().parentHash;
        const storedParentHash = (
          (await apiAtBadBlock.query.ethereum.currentBlock()).unwrap() as any
        ).header.parentHash;

        // The stored parentHash is zero-value due to a missing migration in RT1200.
        expect(storedParentHash.isEmpty, "stored parentHash was not empty").to.be.true;
        expect(rpcParentHash.toString()).to.equal(
          "0x0d0fd88778aec08b3a83ce36387dbf130f6f304fc91e9a44c9605eaf8a80ce5d"
        );

        log(
          `Verified ethereum parentHash ${rpcParentHash} for block #${badBlockNumber} in moonbase \
          (at #${atBlockNumber})`
        );
      },
    });
  },
});
