import { expect } from "chai";

import { describeWithMoonbeam, customRequest, createAndFinalizeBlock } from "./util";
import { AbiItem } from "web3-utils";

describeWithMoonbeam("Moonbeam RPC (Gas)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  // Solidity:
  // contract test {
  //   function multiply(uint a) public pure returns(uint d) {return a * 7;}
  // }
  const TEST_CONTRACT_BYTECODE =
    "0x6080604052348015600f57600080fd5b5060ae8061001e6000396000f3fe6080604052348015600f57600080fd" +
    "5b506004361060285760003560e01c8063c6888fa114602d575b600080fd5b605660048036036020811015604157" +
    "600080fd5b8101908080359060200190929190505050606c565b6040518082815260200191505060405180910390" +
    "f35b600060078202905091905056fea265627a7a72315820f06085b229f27f9ad48b2ff3dd9714350c1698a37853" +
    "a30136fa6c5a7762af7364736f6c63430005110032";

  const TEST_CONTRACT_ABI = {
    constant: true,
    inputs: [{ internalType: "uint256", name: "a", type: "uint256" }],
    name: "multiply",
    outputs: [{ internalType: "uint256", name: "d", type: "uint256" }],
    payable: false,
    stateMutability: "pure",
    type: "function",
  } as AbiItem;

  // Those test are ordered. In general this should be avoided, but due to the time it takes
  // to spin up a Moonbeam node, it saves a lot of time.
  const FIRST_CONTRACT_ADDRESS = "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a";

  it("eth_estimateGas for contract creation", async function () {
    expect(
      await context.web3.eth.estimateGas({
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
      })
    ).to.equal(91019);
  });

  it("eth_estimateGas for contract call", async function () {
    const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
      gasPrice: "0x01",
    });

    expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
  });

  it("eth_estimateGas without gas_limit should pass", async function () {
    const contract = new context.web3.eth.Contract([TEST_CONTRACT_ABI], FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
    });

    expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
  });

  // Current gas per second is at 16M.
  const GAS_PER_SECOND = 16_000_000;
  // The real computation is 1_000_000_000_000 / 16_000_000, but we simplify to avoid bigint.
  const GAS_PER_WEIGHT = 1_000_000 / 16;

  // Our weight limit is 500ms.
  const BLOCK_TX_LIMIT = GAS_PER_SECOND * 0.5;

  // Current implementation is limiting block transactions to 0.75% of the block gas limit
  const BLOCK_TX_GAS_LIMIT = BLOCK_TX_LIMIT * 0.75;
  const EXTRINSIC_BASE_COST = 125_000_000 / GAS_PER_WEIGHT; // 125_000_000 Weight per extrinsics

  // Maximum extrinsic weight is taken from the max allowed transaction weight per block,
  // minus the block initialization (10%) and minus the extrinsic base cost.
  const EXTRINSIC_GAS_LIMIT = BLOCK_TX_GAS_LIMIT - BLOCK_TX_LIMIT * 0.1 - EXTRINSIC_BASE_COST;

  it("gas limit should be fine up to the weight limit", async function () {
    const nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
    const goodTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: EXTRINSIC_GAS_LIMIT, // Todo: fix (remove eth base cost)
        nonce,
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    let resp = await customRequest(context.web3, "eth_sendRawTransaction", [goodTx.rawTransaction]);
    expect(resp.result).to.be.length(66);
  });

  it("gas limit should be limited by weight", async function () {
    const nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
    const badTx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: EXTRINSIC_GAS_LIMIT + 1,
        nonce: nonce,
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    expect(
      ((await customRequest(context.web3, "eth_sendRawTransaction", [badTx.rawTransaction]))
        .error as any).message
    ).to.equal(
      "submit transaction to pool failed: " +
        "Pool(InvalidTransaction(InvalidTransaction::ExhaustsResources))"
    );
  });
});
