import { expect } from "chai";

import { describeWithMoonbeam } from "./util";

describeWithMoonbeam("Moonbeam RPC (Version)", `simple-specs.json`, (context) => {
  it("eth_chainId should match", async function () {
    expect(await context.web3.eth.getChainId()).to.equal(1281);
  });
  it("net_version should match", async function () {
    expect(await context.web3.eth.net.getId()).to.equal(1281);
  });
});
