import { deployContractByName } from "../tests/util/contracts";
import { test } from "../util/setup";

test.only("should be initialized", async (t) => {
  const contract = (
    await t.context.createBlockWith(() => t.context.deployContract("TestContractIncr"))
  ).result;

  t.is(await contract.methods.count().call(), "0");
});

test("should increase smart contract count", async (t) => {
  const contract = (
    await t.context.createBlockWith(() => t.context.deployContract("TestContractIncr"))
  ).result;

  await t.context.createBlockWith(() => contract.methods.incr().call());

  t.is(await contract.methods.count().call(), "1");
});

test("infinite loop call should return OutOfGas", async (t) => {
  const contract = (
    await t.context.createBlockWith(() => t.context.deployContract("InfiniteContract"))
  ).result;

  const error = await t.throwsAsync(contract.methods.infinite().call({ gas: "0x100000" }));

  t.is(error.message, `Returned error: out of gas or fund`);
});

test("inifinite loop send with incr should return OutOfGas", async (t) => {
  const contract = (
    await t.context.createBlockWith(() => t.context.deployContract("InfiniteContractVar"))
  ).result;

  await contract.methods.infinite().call();
  await t.context.createAndFinalizeBlock();

  let block = await t.context.web3.eth.getBlock("latest");
  const receipt = await t.context.web3.eth.getTransactionReceipt(block.transactions[0]);
  t.false(receipt.status);
});

[
  {
    loop: 1,
    gas: 42889,
  },
  {
    loop: 500,
    gas: 1045154,
  },
  {
    loop: 600,
    gas: 1048576,
  },
].forEach(({ loop, gas }) => {
  test(`should consume ${gas} for ${loop} loop`, async (t) => {
    const contract = (
      await t.context.createBlockWith(() => t.context.deployContract("FiniteLoopContract"))
    ).result;

    await t.context.createBlockWith(() => {
      return contract.methods.incr(loop).call();
    });

    t.is(await contract.methods.count().call(), loop);
    let block = await t.context.web3.eth.getBlock("latest");
    t.is(block.gasUsed, gas);
  });
});
