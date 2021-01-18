import { expect } from "chai";
import { describeWithMoonbeam, customRequest } from "./util";

describeWithMoonbeam("Moonbeam RPC (Web3Api)", `simple-specs.json`, (context) => {
  it("should get client version", async function () {
    const version = await context.web3.eth.getNodeInfo();
    let specName: string = await context.polkadotApi.runtimeVersion.specName.toString();
    let specVersion: string = await context.polkadotApi.runtimeVersion.specVersion.toString();
    let implVersion: string = await context.polkadotApi.runtimeVersion.implVersion.toString();
    let regex = new RegExp(specName + "/v" + specVersion + "." + implVersion + "/fc-rpc-0.1.0");
    expect(version).to.be.match(regex);
  });

  it("should remote sha3", async function () {
    const data = context.web3.utils.stringToHex("hello");
    const hash = await customRequest(context.web3, "web3_sha3", [data]);
    const local_hash = context.web3.utils.sha3("hello");
    expect(hash.result).to.be.equal(local_hash);
  });
});
