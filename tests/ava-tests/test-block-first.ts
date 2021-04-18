import { test } from "../util/setup";

test("should be at block 1 after block production", async (t) => {
  await t.context.createAndFinalizeBlock();
  t.is(await t.context.web3.eth.getBlockNumber(), 1);
});

test("should have valid timestamp after block production", async (t) => {
  await t.context.createAndFinalizeBlock();

  // Originally ,this test required the timestamp be in the last five minutes.
  // This requirement doesn't make sense when we forge timestamps in manual seal.
  const block = await t.context.web3.eth.getBlock("latest");
  const next5Minutes = Date.now() / 1000 + 300;
  t.true(block.timestamp > 0);
  t.true(block.timestamp < next5Minutes);
});

test("retrieve block information", async (t) => {
  await t.context.createAndFinalizeBlock();
  const block = await t.context.web3.eth.getBlock("latest");
  t.like(block, {
    author: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
    difficulty: "0",
    extraData: "0x",
    gasLimit: 15000000,
    gasUsed: 0,
    //hash: "0x14fe6f7c93597f79b901f8b5d7a84277a90915b8d355959b587e18de34f1dc17",
    logsBloom: `0x${"0".repeat(512)}`,
    miner: "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b",
    number: 1,
    //parentHash: "0x04540257811b46d103d9896e7807040e7de5080e285841c5430d1a81588a0ce4",
    receiptsRoot: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
    sha3Uncles: "0x1dcc4de8dec75d7aab85b567b6ccd41ad312451b948a7413f0a142fd40d49347",
    totalDifficulty: "0",
    //transactions: [],
    transactionsRoot: "0x56e81f171bcc55a6ff8345e692c0f86e5b48e01b996cadc001622fb5e363b421",
    //uncles: []
  });
  //   previousBlock = block;

  t.true(Array.isArray(block.transactions));
  t.true(Array.isArray(block.uncles));
  t.deepEqual((block as any).sealFields, [
    "0x0000000000000000000000000000000000000000000000000000000000000000",
    "0x0000000000000000",
  ]);
  t.is(block.hash.length, 66);
  t.is(block.parentHash.length, 66);
  t.is(typeof block.timestamp, "number");
});

test("get block by hash", async (t) => {
  await t.context.createAndFinalizeBlock();
  const latestBlock = await t.context.web3.eth.getBlock("latest");
  const block = await t.context.web3.eth.getBlock(latestBlock.hash);
  t.is(block.hash, latestBlock.hash);
});

test("get block by number", async (t) => {
  await t.context.createAndFinalizeBlock();
  const block = await t.context.web3.eth.getBlock(1);
  t.truthy(block);
});
