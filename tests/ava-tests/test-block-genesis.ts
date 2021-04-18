import { test } from "../util/setup";

test("should be at block 0 at genesis", async (t) => {
  t.is(await t.context.web3.eth.getBlockNumber(), 0);
});

test("should return genesis block", async (t) => {
  const block = await t.context.web3.eth.getBlock(0);

  t.is(await t.context.web3.eth.getBlockNumber(), 0);
  t.like(block, {
    author: "0x0000000000000000000000000000000000000000",
    difficulty: "0",
    extraData: "0x",
    gasLimit: 15000000,
    gasUsed: 0,
    logsBloom: `0x${"0".repeat(512)}`,
    number: 0,
    receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
    sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
    totalDifficulty: "0",
    transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  });

  t.true(Array.isArray(block.transactions));
  t.is(block.transactions.length, 0);
  t.true(Array.isArray(block.uncles));
  t.is(block.uncles.length, 0);
  t.deepEqual((block as any).sealFields, [
    "0x0000000000000000000000000000000000000000000000000000000000000000",
    "0x0000000000000000",
  ]);
  t.is(block.hash.length, 66);
  t.is(block.parentHash.length, 66);
  t.is(typeof block.timestamp, "number");
});

test("fetch genesis block by hash", async (t) => {
  //fetch block again using hash
  await t.context.createAndFinalizeBlock();

  const block = await t.context.web3.eth.getBlock(0);
  const blockByHash = await t.context.web3.eth.getBlock(block.hash);
  t.like(blockByHash, {
    author: "0x0000000000000000000000000000000000000000",
    difficulty: "0",
    extraData: "0x",
    gasLimit: 15000000,
    gasUsed: 0,
    logsBloom: `0x${"0".repeat(512)}`,
    number: 0,
    receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
    sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
    totalDifficulty: "0",
    transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
  });
});
