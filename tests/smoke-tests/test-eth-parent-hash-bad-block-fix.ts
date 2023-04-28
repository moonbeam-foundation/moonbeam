import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import chalk from "chalk";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";

const debug = require("debug")("smoke:ethereum-contract");

describeSmokeSuite(
  "S570",
  `Moonbase parentHash fix for block #1648995`,
  async function (context, testIt) {
    let atBlockNumber: number = 0;
    let apiAt: ApiDecoration<"promise"> = null;

    before("configure api at block", async function () {
      this.timeout(6_000_000); // 30 minutes

      atBlockNumber = 1648995;
      apiAt = await context.polkadotApi.at(
        await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
      );
    });

    testIt("C100", `should return correct parent hash via rpc`, async function () {
      const specName = context.polkadotApi.consts.system.version.specName.toString();
      if (specName !== "moonbase") {
        debug(`Test only applies for "moonbase", skipping for "${specName}"`);
        this.skip();
      }

      const rpcParentHash = (
        await context.polkadotApi.rpc.eth.getBlockByNumber(atBlockNumber, false)
      ).unwrap().parentHash;
      const storedParentHash = ((await apiAt.query.ethereum.currentBlock()).unwrap() as any).header
        .parentHash;

      // The stored parentHash is zero-value due to a missing migration in RT1200.
      expect(storedParentHash.isEmpty, "stored parentHash was not empty").to.be.true;
      expect(rpcParentHash.toString()).to.equal(
        "0x0d0fd88778aec08b3a83ce36387dbf130f6f304fc91e9a44c9605eaf8a80ce5d"
      );

      debug(
        `Verified ethereum parentHash ${rpcParentHash} for block in moonbase (at #${atBlockNumber})`
      );
    });
  }
);
