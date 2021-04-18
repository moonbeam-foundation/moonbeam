import { expect } from "chai";
import { FIRST_CONTRACT_ADDRESS, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Moonbeam RPC (Contract)", `simple-specs.json`, (context) => {
  // Those test are ordered. In general this should be avoided, but due to the time it takes
  // to spin up a Moonbeam node, it saves a lot of time.

  it("contract creation should return transaction hash", async function () {
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: (await getCompiled("TestContract")).byteCode,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    expect(
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result: "0x286fc7f456a452abb22bc37974fe281164e53ce6381583c8febaa89c92f31c0b",
    });

    // Verify the contract is not yet stored
    expect(
      await customRequest(context.web3, "eth_getCode", [FIRST_CONTRACT_ADDRESS])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result: "0x",
    });

    // Verify the contract is stored after the block is produced
    await createAndFinalizeBlock(context.polkadotApi);
    expect(
      await customRequest(context.web3, "eth_getCode", [FIRST_CONTRACT_ADDRESS])
    ).to.deep.equal({
      id: 1,
      jsonrpc: "2.0",
      result:
        "0x608060405234801561001057600080fd5b506004361061002b5760003560e01c8063c6888fa114610030" +
        "575b600080fd5b61004a6004803603810190610045919061008b565b610060565b60405161005791906100" +
        "c3565b60405180910390f35b600060078261006f91906100de565b9050919050565b600081359050610085" +
        "81610171565b92915050565b60006020828403121561009d57600080fd5b60006100ab8482850161007656" +
        "5b91505092915050565b6100bd81610138565b82525050565b60006020820190506100d860008301846100" +
        "b4565b92915050565b60006100e982610138565b91506100f483610138565b9250817fffffffffffffffff" +
        "ffffffffffffffffffffffffffffffffffffffffffffffff048311821515161561012d5761012c61014256" +
        "5b5b828202905092915050565b6000819050919050565b7f4e487b71000000000000000000000000000000" +
        "00000000000000000000000000600052601160045260246000fd5b61017a81610138565b81146101855760" +
        "0080fd5b5056fea2646970667358221220b88e1021b77279e6fa68a88b71ca091bb5aa25d4d619327607c4" +
        "e207c587f1e264736f6c63430008030033",
    });
  });
});
