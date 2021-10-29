import { expect } from "chai";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";
import { createContract } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeam("Block Gas - Limit", (context) => {
  it("should be allowed to the max block gas", async function () {
    const { rawTx } = await createContract(context.web3, "TestContract", {
      gas: EXTRINSIC_GAS_LIMIT,
    });
    const { txResults } = await context.createBlock({ transactions: [rawTx] });
    expect(txResults[0].result).to.not.be.null;

    const receipt = await context.web3.eth.getTransaction(txResults[0].result);
    expect(receipt.blockHash).to.be.not.null;
  });
});

describeDevMoonbeam("Block Gas - Limit", (context) => {
  it("should fail setting it over the max block gas", async function () {
    const { rawTx } = await createContract(context.web3, "TestContract", {
      gas: EXTRINSIC_GAS_LIMIT + 1,
    });

    expect(
      ((await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).error as any)
        .message
    ).to.equal(
      "submit transaction to pool failed: " +
        "Pool(InvalidTransaction(InvalidTransaction::ExhaustsResources))"
    );
  });
});

describeDevMoonbeam("Block Gas - Limit", (context) => {
  // TODO: Joshy to fix block gas access in smart contract
  it.skip("should be accessible within a contract", async function () {
    const { contract, rawTx } = await createContract(context.web3, "CheckBlockVariables");
    await context.createBlock({ transactions: [rawTx] });

    expect((await contract.methods.gaslimit().call()) !== "0").to.eq(true);
  });
});
