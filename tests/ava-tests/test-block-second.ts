import { test } from "../util/setup";

test("should include previous block hash as parent", async (t) => {
  await t.context.createAndFinalizeBlock();
  await t.context.createAndFinalizeBlock();
  const block = await t.context.web3.eth.getBlock("latest");
  let previousBlock = await t.context.web3.eth.getBlock(1);
  t.not(block.hash, previousBlock.hash);
  t.is(block.parentHash, previousBlock.hash);
});
