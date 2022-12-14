import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { expectEVMResult } from "../../util/eth-transactions";

import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeam, describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

describeDevMoonbeamAllEthTxTypes("Precompiles - ripemd160 ", (context) => {
  it("should be valid", async function () {
    expect(
      (
        await web3EthCall(context.web3, {
          to: "0x0000000000000000000000000000000000000003",
          data: `0x${Buffer.from("Hello world!").toString("hex")}`,
        })
      ).result
    ).equals("0x0000000000000000000000007f772647d88750add82d8e1a7a3e5c0902a346a3");
  });
});

describeDevMoonbeam("Precompiles - ripemd160 ", (context) => {
  it("should be accessible from a smart contract", async function () {
    // Deploy the contract
    const { contract, rawTx } = await createContract(context, "HasherChecker");
    await context.createBlock(rawTx);

    // Execute the contract ripemd160 call
    const { result } = await context.createBlock(
      createContractExecution(context, {
        contract,
        contractCall: contract.methods.ripemd160Check(),
      })
    );

    // Verify the result
    expectEVMResult(result.events, "Succeed");
  });
});
