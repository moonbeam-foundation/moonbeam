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
