import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";

import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";
import { createContract } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";

describeDevMoonbeamAllEthTxTypes("Block Gas - Limit", (context) => {
  it("should be allowed to the max block gas", async function () {
    const { rawTx } = await createContract(context, "TestContract", { gas: EXTRINSIC_GAS_LIMIT });
    const { result } = await context.createBlockWithEth(rawTx);
    expect(result.result).to.not.be.null;

    const receipt = await context.web3.eth.getTransaction(result.result);
    expect(receipt.blockHash).to.be.not.null;
  });
});

describeDevMoonbeamAllEthTxTypes("Block Gas - Limit", (context) => {
  it("should fail setting it over the max block gas", async function () {
    const { rawTx } = await createContract(context, "TestContract", {
      gas: EXTRINSIC_GAS_LIMIT + 1,
    });

    expect(
      ((await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).error as any)
        .message
    ).to.equal("gas limit reached");
  });
});

describeDevMoonbeam("Block Gas - Limit", (context) => {
  // TODO: Joshy to fix block gas access in smart contract
  it.skip("should be accessible within a contract", async function () {
    const { contract, rawTx } = await createContract(context, "CheckBlockVariables");
    await context.createBlockWithEth(rawTx);
    expect((await contract.methods.gaslimit().call()) !== "0").to.eq(true);
  });
});
