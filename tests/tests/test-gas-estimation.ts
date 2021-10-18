import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";

import { describeDevMoonbeam } from "../util/setup-dev-tests";

import { createContract } from "../util/transactions";
import { Contract } from "web3-eth-contract";

chaiUse(chaiAsPromised);

describeDevMoonbeam("Estimate Gas - Multiply", (context) => {
  let multContract: Contract;

  before("Setup: Create simple context", async function () {
    const { contract, rawTx } = await createContract(context.web3, "TestContract");
    await context.createBlock({ transactions: [rawTx] });
    multContract = contract;
  });

  it("should return correct gas estimation", async function () {
    expect(await multContract.methods.multiply(3).estimateGas()).to.equal(22405);
  });

  it("should work without gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: null,
      })
    ).to.equal(22405);
  });

  it("should work with gas limit", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        gas: 22405,
      })
    ).to.lessThanOrEqual(22405);
  });

  it("should ignore from balance (?)", async function () {
    expect(
      await multContract.methods.multiply(3).estimateGas({
        from: "0x0000000000000000000000000000000000000000",
      })
    ).to.equal(22405);
  });

  it("should not work with a lower gas limit", async function () {
    await expect(
      multContract.methods.multiply(3).estimateGas({
        gas: 21900,
      })
    ).to.be.rejectedWith("gas required exceeds allowance 21900");
  });
});
