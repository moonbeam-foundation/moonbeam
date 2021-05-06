import { expect } from "chai";
import { customWeb3Request } from "../util/providers";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

describeDevMoonbeam("Web3Api Information", (context) => {
  it("should include client version", async function () {
    const version = await context.web3.eth.getNodeInfo();
    let specName: string = await context.polkadotApi.runtimeVersion.specName.toString();
    let specVersion: string = await context.polkadotApi.runtimeVersion.specVersion.toString();
    let implVersion: string = await context.polkadotApi.runtimeVersion.implVersion.toString();
    let regex = new RegExp(specName + "/v" + specVersion + "." + implVersion + "/fc-rpc-2.0.0");
    expect(version).to.be.match(regex);
  });

  it("should provide sha3 hashing", async function () {
    const data = context.web3.utils.stringToHex("hello");
    const hash = await customWeb3Request(context.web3, "web3_sha3", [data]);
    const localhash = context.web3.utils.sha3("hello");
    expect(hash.result).to.be.equal(localhash);
  });

  it("should report peer count in hex", async function () {
    // this tests that the "net_peerCount" response comes back in hex and not decimal.
    // this seems a bit inconsistent amongst Ethereum APIs, but hex seems to be most common.

    // related: frontier commits 677548c and 78fb3bc

    const result = await customWeb3Request(context.web3, "net_peerCount", []);

    // TODO: this is really just testing that the result comes back as a string, not that it's
    //       expressed in hex (as opposed to decimal)
    expect(result.result).to.be.equal("0x0");
    expect(typeof result.result).to.be.equal("string");
  });
});
