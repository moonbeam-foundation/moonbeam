import "@moonbeam-network/api-augment";
import { nToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";
import { alith, baltathar } from "../../util/accounts";
import { MIN_GLMR_DELEGATOR, PRECOMPILE_PARACHAIN_STAKING_ADDRESS } from "../../util/constants";
import { getCompiled } from "../../util/contracts";
import { expectEVMResult } from "../../util/eth-transactions";
import { expectOk } from "../../util/expect";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";

const SMART_CONTRACT_PRECOMPILE_CALL_JSON = getCompiled("SmartContractPrecompileCallTest");
const SMART_CONTRACT_PRECOMPILE_CALL = new ethers.utils.Interface(
  SMART_CONTRACT_PRECOMPILE_CALL_JSON.contract.abi
);

const MULTIPLY_BY_7 = new ethers.utils.Interface(getCompiled("MultiplyBy7").contract.abi);
const PARACHAIN_STAKING = new ethers.utils.Interface(
  getCompiled("precompiles/parachain-staking/ParachainStaking").contract.abi
);

async function setupTestContract(context: DevTestContext) {
  const { contract, rawTx } = await createContract(context, "SmartContractPrecompileCallTest", {
    ...ALITH_TRANSACTION_TEMPLATE,
    gas: 5_000_000,
  });
  const { result } = await context.createBlock(rawTx);
  expectEVMResult(result.events, "Succeed");

  return contract;
}

async function setupMultiplyContract(context: DevTestContext) {
  const { contract, rawTx } = await createContract(context, "MultiplyBy7", {
    ...ALITH_TRANSACTION_TEMPLATE,
    gas: 5_000_000,
  });
  const { result } = await context.createBlock(rawTx);
  expectEVMResult(result.events, "Succeed");

  return contract;
}

describeDevMoonbeam("Smart Contract Precompile Call - AddProxy", (context) => {
  let contract: Contract;
  before("setup contract", async function () {
    contract = await setupTestContract(context);
  });

  it("should revert when caller is a smart contract", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callAddProxy", [
          baltathar.address,
        ]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });
});

describeDevMoonbeam("Smart Contract Precompile Call - Proxy - Correct Proxy Type", (context) => {
  let contract: Contract;
  before("setup contract and Staking proxy", async function () {
    contract = await setupTestContract(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.proxy
          .addProxy(contract.options.address, "Staking", 0)
          .signAsync(baltathar)
      )
    );
  });

  it("should succeed when caller is a smart contract", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callProxy", [
          baltathar.address,
          PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
          PARACHAIN_STAKING.encodeFunctionData("delegateWithAutoCompound", [
            alith.address,
            MIN_GLMR_DELEGATOR,
            100,
            0,
            0,
            0,
          ]),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
    const delegations = await context.polkadotApi.query.parachainStaking.topDelegations(
      alith.address
    );
    expect(delegations.toJSON()).to.deep.equal({
      delegations: [
        {
          owner: baltathar.address,
          amount: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
        },
      ],
      total: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
    });
  });
});

describeDevMoonbeam("Smart Contract Precompile Call - Proxy - Any Proxy Type", (context) => {
  let contract: Contract;
  before("setup contract and Any proxy", async function () {
    contract = await setupTestContract(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.proxy
          .addProxy(contract.options.address, "Any", 0)
          .signAsync(baltathar)
      )
    );
  });

  it("should succeed when caller is a smart contract", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callProxy", [
          baltathar.address,
          PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
          PARACHAIN_STAKING.encodeFunctionData("delegateWithAutoCompound", [
            alith.address,
            MIN_GLMR_DELEGATOR,
            100,
            0,
            0,
            0,
          ]),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
    const delegations = await context.polkadotApi.query.parachainStaking.topDelegations(
      alith.address
    );
    expect(delegations.toJSON()).to.deep.equal({
      delegations: [
        {
          owner: baltathar.address,
          amount: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
        },
      ],
      total: nToHex(MIN_GLMR_DELEGATOR, { bitLength: 128 }),
    });
  });
});

describeDevMoonbeam("Smart Contract Precompile Call - Proxy - Incorrect Proxy Type", (context) => {
  let contract: Contract;
  before("setup contract and governance proxy", async function () {
    contract = await setupTestContract(context);
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.proxy
          .addProxy(contract.options.address, "Governance", 0)
          .signAsync(baltathar)
      )
    );
  });

  it("should revert when caller is a smart contract", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callProxy", [
          baltathar.address,
          PRECOMPILE_PARACHAIN_STAKING_ADDRESS,
          PARACHAIN_STAKING.encodeFunctionData("delegateWithAutoCompound", [
            alith.address,
            MIN_GLMR_DELEGATOR,
            100,
            0,
            0,
            0,
          ]),
        ]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });
});

describeDevMoonbeam("Smart Contract Precompile Call - Proxy - Real Account", (context) => {
  let contract: Contract;
  let otherContract: Contract;
  let multiplyContract: Contract;
  before("setup contract and Any proxy", async function () {
    contract = await setupTestContract(context);
    otherContract = await setupTestContract(context);
    multiplyContract = await setupMultiplyContract(context);

    // Add proxy from baltathar to the test smart contract
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.proxy
          .addProxy(contract.options.address, "Any", 0)
          .signAsync(baltathar)
      )
    );

    // Add proxy from a canary smart contract to the test smart contract via setStorage
    const storageKey = context.polkadotApi.query.proxy.proxies.key(otherContract.options.address);
    const storageValue = context.polkadotApi.registry
      .createType("(Vec<PalletProxyProxyDefinition>,u128)", [
        [
          {
            delegate: contract.options.address,
            proxyType: "Any",
            delay: 0,
          },
        ],
        1002900000000000000n,
      ])
      .toHex();
    await expectOk(
      context.createBlock(
        context.polkadotApi.tx.sudo.sudo(
          context.polkadotApi.tx.system.setStorage([[storageKey, storageValue]])
        )
      )
    );
  });

  it("should revert when caller is a smart contract and real address is \
smart contract", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callProxy", [
          otherContract.options.address,
          multiplyContract.options.address,
          MULTIPLY_BY_7.encodeFunctionData("multiply", [5]),
        ]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });

  it("should succeed when caller is a smart contract and real address is EOA", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callProxy", [
          baltathar.address,
          multiplyContract.options.address,
          MULTIPLY_BY_7.encodeFunctionData("multiply", [5]),
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });
});

describeDevMoonbeam("Smart Contract Precompile Call - Batch", (context) => {
  let contract: Contract;
  let multiplyContract: Contract;
  before("setup contract", async function () {
    contract = await setupTestContract(context);
    multiplyContract = await setupMultiplyContract(context);
  });

  it("should revert when caller is a smart contract", async function () {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 10_000_000,
        to: contract.options.address,
        data: SMART_CONTRACT_PRECOMPILE_CALL.encodeFunctionData("callBatch", [
          multiplyContract.options.address,
          [
            MULTIPLY_BY_7.encodeFunctionData("multiply", [5]),
            MULTIPLY_BY_7.encodeFunctionData("multiply", [6]),
            MULTIPLY_BY_7.encodeFunctionData("multiply", [7]),
          ],
        ]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });
});
