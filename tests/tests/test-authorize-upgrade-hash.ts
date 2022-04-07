import { expect } from "chai";
import { blake2AsHex } from "@polkadot/util-crypto";

const MOONBASE_RUNTIME_701_BLAKE2_256 =
  "d541cb056f0611bc0fc36b3371cd11d705924b4bfa5c41840111fd2e8cf37a3f";
const MOONBASE_RUNTIME_701_AUTHORIZE_UPGRADE_SRTOOL =
  "0x64cf767488b73d7cc5281ebd6f0a2db15cb1f90076babe0fe0084ec8c5620c9b";
const MOONBASE_RUNTIME_701_AUTHORIZE_UPGRADE_REAL =
  "0xf4c35abcb96d3d870db495d63a12c734911ca8cf0279c36f58ddeab63e225079";

describe("Compute AuthorizeUpgrade hash", () => {
  it("Srtool should compute with prefix 0x0103", function () {
    let authorizeUpgradeHash1 = blake2AsHex("0x0103" + MOONBASE_RUNTIME_701_BLAKE2_256);
    expect(authorizeUpgradeHash1).to.equal(MOONBASE_RUNTIME_701_AUTHORIZE_UPGRADE_SRTOOL);
  });
  it("Real hash should be computed with prefix 0x0603", function () {
    let authorizeUpgradeHash6 = blake2AsHex("0x0603" + MOONBASE_RUNTIME_701_BLAKE2_256);
    expect(authorizeUpgradeHash6).to.equal(MOONBASE_RUNTIME_701_AUTHORIZE_UPGRADE_REAL);
  });
});
