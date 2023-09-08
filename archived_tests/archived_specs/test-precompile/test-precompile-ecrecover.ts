import "@moonbeam-network/api-augment";
import { ethers, Contract } from "ethers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { ALITH_ADDRESS, ALITH_PRIVATE_KEY } from "../../util/accounts";
import { expect } from "chai";
import { getCompiled } from "../../util/contracts";
import { u8aToHex, u8aToString } from "@polkadot/util";

describeDevMoonbeam("Precompiles - ecrecover", (context) => {
  let ethersContract: Contract;

  before(async function () {
    const { rawTx, contractAddress } = await createContract(context, "RecoveryChecker");
    await context.createBlock(rawTx);

    const contractJson = getCompiled("RecoveryChecker");
    const contractAbi = new ethers.utils.Interface(contractJson.contract.abi);

    ethersContract = new ethers.Contract(contractAddress, contractAbi, context.ethers);
  });

  it("returns a matching address", async function () {
    const msg = context.web3.utils.sha3("Hello World!");
    const signed = context.web3.eth.accounts.sign(msg, ALITH_PRIVATE_KEY);
    const result = await ethersContract.checkRecovery(
      signed.messageHash,
      signed.v,
      signed.r,
      signed.s
    );
    expect(result, "Recovered address doesn't match signer!").to.equals(ALITH_ADDRESS);
  });

  it("returns different address on modified message", async function () {
    const msg = context.web3.utils.sha3("Hello World!");
    const signed = context.web3.eth.accounts.sign(msg, ALITH_PRIVATE_KEY);
    const result = await ethersContract.checkRecovery(
      signed.messageHash.replace("1", "f"),
      signed.v,
      signed.r,
      signed.s
    );
    expect(result, "Recovered address doesn't match signer!").to.equals(
      "0x58188b9AE77F7C865b04B12F5D29bF4fbDcbd937"
    );
  });

  it("returns empty on invalid V", async function () {
    const msg = context.web3.utils.sha3("Hello World!");
    const signed = context.web3.eth.accounts.sign(msg, ALITH_PRIVATE_KEY);
    const v = "0x565656ff5656ffffffffffff3d3d02000000000040003dffff565656560f0000";
    try {
      const result = await ethersContract.checkRecovery(signed.messageHash, v, signed.r, signed.s);
      expect(result).to.equal("0x0000000000000000000000000000000000000000");
    } catch (e) {
      console.error(e);
      expect(e.data, "Empty data field should be returned").to.equal("0x");
    }
  });
});
