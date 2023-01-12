import "@moonbeam-network/api-augment";
import { Contract } from "web3-eth-contract";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { ALITH_ADDRESS, ALITH_PRIVATE_KEY } from "../../util/accounts";
import { expect } from "chai";

describeDevMoonbeam("Precompiles - ecrecover", (context) => {
  let recoverContract: Contract;

  before(async function () {
    const { contract, rawTx } = await createContract(context, "RecoveryChecker");
    await context.createBlock(rawTx);
    recoverContract = contract;
  });

  it("returns a matching address", async function () {
    const msg = context.web3.utils.sha3("Hello World!");
    const signed = context.web3.eth.accounts.sign(msg, ALITH_PRIVATE_KEY);
    const result = await recoverContract.methods
      .checkRecovery(signed.messageHash, signed.v, signed.r, signed.s)
      .call();
    expect(result, "Recovered address doesn't match signer!").to.equals(ALITH_ADDRESS);
  });

  it("returns different address on modified message", async function () {
    const msg = context.web3.utils.sha3("Hello World!");
    const signed = context.web3.eth.accounts.sign(msg, ALITH_PRIVATE_KEY);
    const result = await recoverContract.methods
      .checkRecovery(signed.messageHash.replace("1", "f"), signed.v, signed.r, signed.s)
      .call();
    expect(result, "Recovered address doesn't match signer!").to.equals(
      "0x58188b9AE77F7C865b04B12F5D29bF4fbDcbd937"
    );
  });

  it("returns empty on invalid V", async function () {
    const msg = context.web3.utils.sha3("Hello World!");
    const signed = context.web3.eth.accounts.sign(msg, ALITH_PRIVATE_KEY);

    const v = "0x565656ff5656ffffffffffff3d3d02000000000040003dffff565656560f0000";
    const result = await recoverContract.methods
      .checkRecovery(signed.messageHash, v, signed.r, signed.s)
      .call();
    expect(result, "Precompile should return zero for invalid V").to.equals(
      "0x0000000000000000000000000000000000000000"
    );
  });
});
