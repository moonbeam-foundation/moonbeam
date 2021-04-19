import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { createContract } from "../util/transactions";

describeDevMoonbeam("Version RPC", (context) => {
  it("should return 1281 for eth_chainId", async function () {
    expect(await context.web3.eth.getChainId()).to.equal(1281);
  });
  it("should return 1281 for net_version", async function () {
    expect(await context.web3.eth.net.getId()).to.equal(1281);
  });
});

describeDevMoonbeam("Version - ChainId", (context) => {
  it("should be accessible within a contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "CheckBlockGasLimit");
    await context.createBlock({ transactions: [rawTx] });

    expect(await contract.methods.chainid().call()).to.equal("1281");
  });
});
