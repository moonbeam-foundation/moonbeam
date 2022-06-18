import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";

describeDevMoonbeam("Web3Api Information", (context) => {
  it("should include client version", async function () {
    const version = await context.web3.eth.getNodeInfo();
    let specName: string = await context.polkadotApi.runtimeVersion.specName.toString();
    let specVersion: string = await context.polkadotApi.runtimeVersion.specVersion.toString();
    let implVersion: string = await context.polkadotApi.runtimeVersion.implVersion.toString();
    let regex = new RegExp(specName + "/v" + specVersion + "." + implVersion + "/fc-rpc-2.0.0");
    expect(version).to.be.match(regex);
  });
});
