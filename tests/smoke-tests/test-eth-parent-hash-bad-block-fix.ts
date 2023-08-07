import "@moonbeam-network/api-augment";
import { ApiDecoration } from "@polkadot/api/types";
import { expect } from "chai";
import { describeSmokeSuite } from "../util/setup-smoke-tests";
import { THIRTY_MINS } from "../util/constants";

const debug = require("debug")("smoke:ethereum-block-fix");

describeSmokeSuite("S570", `RPC Eth ParentHash`, async function (context, testIt) {
  let atBlockNumber: number = 0;
  let previousBlockNumber: number = 0;
  let apiAt: ApiDecoration<"promise">;
  let apiAtPrevious: ApiDecoration<"promise"> = null;

  before("configure api at block", async function () {
    this.timeout(THIRTY_MINS);

    atBlockNumber = (await context.polkadotApi.rpc.chain.getHeader()).number.toNumber();
    previousBlockNumber = atBlockNumber - 1;
    apiAt = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber)
    );
    apiAtPrevious = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(atBlockNumber - 1)
    );
  });

  testIt("C100", `should return correct parent hash via rpc for current block`, async function () {
    const rpcParentHash = (
      await context.polkadotApi.rpc.eth.getBlockByNumber(atBlockNumber, false)
    ).unwrap().parentHash;
    const storedParentHash = ((await apiAt.query.ethereum.currentBlock()).unwrap() as any).header
      .parentHash;

    const actualParentHash = (
      await context.polkadotApi.rpc.eth.getBlockByNumber(previousBlockNumber, false)
    ).unwrap().blockHash;
    expect(storedParentHash.isEmpty, "stored parentHash was empty").to.be.false;
    expect(rpcParentHash.toString()).to.equal(actualParentHash.toString());

    debug(
      `Verified ethereum parentHash ${rpcParentHash} for block #${atBlockNumber} \
          (at #${atBlockNumber})`
    );
  });

  testIt("C200", `should return correct parent hash via rpc for block #1648995`, async function () {
    const badBlockNumber = 1648995;
    const apiAtBadBlock = await context.polkadotApi.at(
      await context.polkadotApi.rpc.chain.getBlockHash(badBlockNumber)
    );

    const chainName = (await context.polkadotApi.rpc.system.chain()).toString();
    if (chainName !== "Moonbase Alpha") {
      debug(`Test only applies for "Moonbase Alpha", skipping for "${chainName}"`);
      this.skip();
    }

    const rpcParentHash = (
      await context.polkadotApi.rpc.eth.getBlockByNumber(badBlockNumber, false)
    ).unwrap().parentHash;
    const storedParentHash = ((await apiAtBadBlock.query.ethereum.currentBlock()).unwrap() as any)
      .header.parentHash;

    // The stored parentHash is zero-value due to a missing migration in RT1200.
    expect(storedParentHash.isEmpty, "stored parentHash was not empty").to.be.true;
    expect(rpcParentHash.toString()).to.equal(
      "0x0d0fd88778aec08b3a83ce36387dbf130f6f304fc91e9a44c9605eaf8a80ce5d"
    );

    debug(
      `Verified ethereum parentHash ${rpcParentHash} for block #${badBlockNumber} in moonbase \
          (at #${atBlockNumber})`
    );
  });
});
