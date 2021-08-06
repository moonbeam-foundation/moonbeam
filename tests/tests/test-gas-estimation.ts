import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { createContract } from "../util/transactions";
import { Contract } from "web3-eth-contract";

describeDevMoonbeam("Estimate Gas - Multiply", (context) => {
  let multContract: Contract;

  before("Setup: Create simple context", async function () {
    const { contract, rawTx } = await createContract(context.web3, "TestContract");
    await context.createBlock({ transactions: [rawTx] });
    multContract = contract;
  });

  it("should return correct gas estimation", async function () {
    expect(await multContract.methods.multiply(3).estimateGas()).to.equal(21994);
  });

  it("should work without gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: null,
      })
    ).to.equal(21994);
  });

  it("should work with gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: 21994,
      })
    ).to.equal(21994);
  });

  it("should ignore from balance (?)", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: 21994,
      })
    ).to.equal(21994);
  });

  // Since the binary search has been activated, the gas indicated in the request is not taken into
  // account by the estimation, so any provided gas limit should work:
  // https://github.com/PureStake/frontier/blob/moonbeam-polkadot-v0.9.8-binary-search/client/rpc/
  // src/eth.rs#L907
  it("should work with a lower gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: 21993,
      })
    ).to.equal(21994);
  });
});
