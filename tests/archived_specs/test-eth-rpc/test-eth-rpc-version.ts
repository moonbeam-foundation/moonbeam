import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Version RPC", (context) => {
  it("should return 1281 for eth_chainId", async function () {
    expect(await context.web3.eth.getChainId()).to.equal(1281);
  });
  it("should return 1281 for net_version", async function () {
    expect(await context.web3.eth.net.getId()).to.equal(1281);
  });
  it("should include client version", async function () {
    const version = await context.web3.eth.getNodeInfo();
    let specName: string = await context.polkadotApi.runtimeVersion.specName.toString();
    let specVersion: string = await context.polkadotApi.runtimeVersion.specVersion.toString();
    let implVersion: string = await context.polkadotApi.runtimeVersion.implVersion.toString();
    let regex = new RegExp(specName + "/v" + specVersion + "." + implVersion + "/fc-rpc-2.0.0");
    expect(version).to.be.match(regex);
  });
});
