import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { Contract } from "web3-eth-contract";

import { describeDevMoonbeam } from "../../../util/setup-dev-tests";
import { createContract } from "../../../util/transactions";

describeDevMoonbeam("Block Contract - Block variables", (context) => {
  let blockContract: Contract;

  before("Setup: Creating contract with block variables", async function () {
    const { contract, rawTx } = await createContract(context, "BlockVariables");
    await context.createBlock(rawTx);
    blockContract = contract;
  });

  it("should store the valid block number at creation", async function () {
    expect(await blockContract.methods.initialnumber().call()).to.eq("1");
  });

  // TODO: Fix block number from contract call
  it.skip("should return parent block number + 1 when accessed by RPC call", async function () {
    const block = await context.web3.eth.getBlock("latest");
    expect(await blockContract.methods.getNumber().call()).to.eq("1");
    expect(await blockContract.methods.getNumber().call()).to.eq(block.number.toString());
  });

  it("should store the valid chain id at creation", async function () {
    expect(await blockContract.methods.initialchainid().call()).to.equal("1281");
  });
});
