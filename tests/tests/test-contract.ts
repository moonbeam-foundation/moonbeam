import { expect } from "chai";
import {
  FIRST_CONTRACT_ADDRESS,
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  TEST_CONTRACT_BYTECODE,
} from "./constants";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

describeWithMoonbeam("Moonbeam RPC (Contract)", `simple-specs.json`, (context) => {
  // Those test are ordered. In general this should be avoided, but due to the time it takes
  // to spin up a Moonbeam node, it saves a lot of time.

  it("contract creation should return transaction hash", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
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
      result: "0xe87ed993e4d186748404a52a2d13612eef8356331f30fa6b3fb9bc2c16be2e9c",
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
        "0x6080604052348015600f57600080fd5b506004361060285760003560e01c8063c6888fa114602d575b60" +
        "0080fd5b605660048036036020811015604157600080fd5b8101908080359060200190929190505050606c" +
        "565b6040518082815260200191505060405180910390f35b600060078202905091905056fea265627a7a72" +
        "315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853a30136fa6c5a7762af7364736f6c63430005" +
        "110032",
    });
  });
});
