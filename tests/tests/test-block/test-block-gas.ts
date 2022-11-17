import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { EXTRINSIC_GAS_LIMIT } from "../../util/constants";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Block Gas - Limit", (context) => {
  it("should be allowed to the max block gas", async function () {
    const { rawTx } = await createContract(context, "MultiplyBy7", { gas: EXTRINSIC_GAS_LIMIT });
    const { result } = await context.createBlock(rawTx);
    expect(result.successful).to.be.true;

    const receipt = await context.web3.eth.getTransaction(result.hash);
    expect(receipt.blockHash).to.be.not.null;
  });
});

describeDevMoonbeamAllEthTxTypes("Block Gas - Limit", (context) => {
  it("should fail setting it over the max block gas", async function () {
    const { rawTx } = await createContract(context, "MultiplyBy7", {
      gas: EXTRINSIC_GAS_LIMIT + 1,
    });

    expect(
      ((await customWeb3Request(context.web3, "eth_sendRawTransaction", [rawTx])).error as any)
        .message
    ).to.equal("exceeds block gas limit");
  });
});

describeDevMoonbeam("Block Gas - Limit", (context) => {
  // TODO: Joshy to fix block gas access in smart contract
  it("should be accessible within a contract", async function () {
    const { contract, rawTx } = await createContract(context, "BlockVariables");
    await context.createBlock(rawTx);
    expect(await contract.methods.getGasLimit().call()).to.equal("15000000");
  });
});
