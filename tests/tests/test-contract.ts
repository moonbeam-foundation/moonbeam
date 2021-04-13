import { expect } from "chai";
import {
  FIRST_CONTRACT_ADDRESS,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  //TEST_CONTRACT_BYTECODE,
} from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Moonbeam RPC (Contract)", `simple-specs.json`, (context) => {
  // Those test are ordered. In general this should be avoided, but due to the time it takes
  // to spin up a Moonbeam node, it saves a lot of time.

  it("contract creation should return transaction hash", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: (await getCompiled("TEST_CONTRACT")).byteCode,
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
      result: "0x70a1cae10c9e0c4f5824ec9ce4c75a9679d58bd26d2fde958b226d75ee0c37d0",
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
        "0x608060405234801561001057600080fd5b506004361061002b5760003560e01c8063c6888fa11461003057" +
        "5b600080fd5b61004a6004803603810190610045919061008b565b610060565b60405161005791906100c356" +
        "5b60405180910390f35b600060078261006f91906100de565b9050919050565b600081359050610085816101" +
        "71565b92915050565b60006020828403121561009d57600080fd5b60006100ab84828501610076565b915050" +
        "92915050565b6100bd81610138565b82525050565b60006020820190506100d860008301846100b4565b9291" +
        "5050565b60006100e982610138565b91506100f483610138565b9250817fffffffffffffffffffffffffffff" +
        "ffffffffffffffffffffffffffffffffffff048311821515161561012d5761012c610142565b5b8282029050" +
        "92915050565b6000819050919050565b7f4e487b710000000000000000000000000000000000000000000000" +
        "0000000000600052601160045260246000fd5b61017a81610138565b811461018557600080fd5b5056fea264" +
        "6970667358221220a423910697c81c1ace928b799ba136adba7f66301855592bd2f156e072bde1a964736f6c" +
        "63430008030033",
    });
  });
});
