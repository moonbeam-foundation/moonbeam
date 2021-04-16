import { expect } from "chai";

import { describeWithMoonbeam, customRequest } from "./util";
import { FIRST_CONTRACT_ADDRESS, GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";
import { getCompiled } from "./util/contracts";

describeWithMoonbeam("Moonbeam RPC (Gas)", `simple-specs.json`, async (context) => {
  // Those test are ordered. In general this should be avoided, but due to the time it takes
  // to spin up a Moonbeam node, it saves a lot of time.
  let TestContractByteCode: string;
  let TestContractABI;

  before("get constants", async function () {
    this.timeout(15000);
    TestContractByteCode = (await getCompiled("TestContract")).byteCode;
    TestContractABI = (await getCompiled("TestContract")).contract.abi;
  });

  it("eth_estimateGas for contract creation", async function () {
    expect(
      await context.web3.eth.estimateGas({
        from: GENESIS_ACCOUNT,
        data: TestContractByteCode,
      })
    ).to.equal(149143);
  });

  it("eth_estimateGas for contract call", async function () {
    const contract = new context.web3.eth.Contract(TestContractABI, FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
      gasPrice: "0x01",
    });

    expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
  });

  it("eth_estimateGas without gas_limit should pass", async function () {
    const contract = new context.web3.eth.Contract(TestContractABI, FIRST_CONTRACT_ADDRESS, {
      from: GENESIS_ACCOUNT,
    });

    expect(await contract.methods.multiply(3).estimateGas()).to.equal(21204);
  });

  // Current gas per second
  const GAS_PER_SECOND = 40_000_000;
  // The real computation is 1_000_000_000_000 / 40_000_000, but we simplify to avoid bigint.
  const GAS_PER_WEIGHT = 1_000_000 / 40;

  // Our weight limit is 500ms.
  const BLOCK_TX_LIMIT = GAS_PER_SECOND * 0.5;

  // Current implementation is limiting block transactions to 75% of the block gas limit
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
        data: TestContractByteCode,
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
        data: TestContractByteCode,
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
