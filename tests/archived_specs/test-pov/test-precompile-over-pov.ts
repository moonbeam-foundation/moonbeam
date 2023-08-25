import "@moonbeam-network/api-augment";

import { expect, use as chaiUse } from "chai";
import { ethers } from "ethers";
import { Interface } from "ethers/src.ts/utils";
import chaiAsPromised from "chai-as-promised";

import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createTransaction } from "../../util/transactions";
import { PRECOMPILE_BATCH_ADDRESS, MAX_ETH_POV_PER_TX } from "../../util/constants";
import { Contract } from "web3-eth-contract";
import { expectEVMResult } from "../../util/eth-transactions";
import { deployHeavyContracts, HeavyContract } from "./test-evm-over-pov";

chaiUse(chaiAsPromised);

describeDevMoonbeam("PoV precompile test - gasLimit", (context) => {
  let contractProxy: Contract;
  let contracts: HeavyContract[];
  const MAX_CONTRACTS = 50;
  const EXPECTED_POV_ROUGH = 1_000_000; // bytes
  let batchInterface: Interface;
  let proxyInterface: Interface;

  before("Deploy the contracts from range 6000-6050", async function () {
    // Deploy the CallForwarder contract
    const creation = await createContract(context, "CallForwarder");
    contractProxy = creation.contract;
    await context.createBlock(creation.rawTx);

    // Get the CallForwarder contract interface
    proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);

    // Deploy heavy contracts
    contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);

    // Get the interface for Batch precompile
    batchInterface = new ethers.utils.Interface(
      getCompiled("precompiles/batch/Batch").contract.abi
    );
  });

  it("gas cost should have increased with POV", async function () {
    const { result, block } = await context.createBlock(
      // Previously this tx cost was ~500K gas -> now it is about 5M due to POV.
      // We pass 1M, so it should fail.
      createTransaction(context, {
        to: PRECOMPILE_BATCH_ADDRESS,
        data: batchInterface.encodeFunctionData("batchAll", [
          [contractProxy.options.address],
          [],
          [
            proxyInterface.encodeFunctionData("callRange", [
              contracts[0].account,
              contracts[MAX_CONTRACTS].account,
            ]),
          ],
          [],
        ]),
        gas: 1_000_000,
      })
    );

    // With 1M gas we are allowed to use ~250k of POV, so verify the range.
    // The tx is still included in the block because it contains the failed tx,
    // so POV is included in the block as well.
    expect(block.proof_size).to.be.at.least(230_000);
    expect(block.proof_size).to.be.at.most(290_000);
    expect(result.successful).to.equal(true);
    expectEVMResult(result.events, "Error", "OutOfGas");
  });

  it("should be able to create a block using the estimated amount of gas", async function () {
    const { result, block } = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_BATCH_ADDRESS,
        data: batchInterface.encodeFunctionData("batchAll", [
          [contractProxy.options.address],
          [],
          [
            proxyInterface.encodeFunctionData("callRange", [
              contracts[0].account,
              contracts[MAX_CONTRACTS].account,
            ]),
          ],
          [],
        ]),
      })
    );
    expect(block.proof_size).to.be.at.least(EXPECTED_POV_ROUGH / 1.3);
    expect(block.proof_size).to.be.at.most(EXPECTED_POV_ROUGH * 1.3);
    expect(result.successful).to.equal(true);
    expectEVMResult(result.events, "Succeed", "Returned");
  });

  it("should allow to call a precompile tx with enough gas limit to cover PoV", async function () {
    const { result, block } = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_BATCH_ADDRESS,
        data: batchInterface.encodeFunctionData("batchAll", [
          [contractProxy.options.address],
          [],
          [
            proxyInterface.encodeFunctionData("callRange", [
              contracts[0].account,
              contracts[MAX_CONTRACTS].account,
            ]),
          ],
          [],
        ]),
        gas: 6_000_000,
      })
    );
    expect(block.proof_size).to.be.at.least(EXPECTED_POV_ROUGH / 1.3);
    expect(block.proof_size).to.be.at.most(EXPECTED_POV_ROUGH * 1.3);
    expect(result.successful).to.equal(true);
    expectEVMResult(result.events, "Succeed", "Returned");
  });
});

describeDevMoonbeam("PoV precompile test - PoV Limit (3.5Mb in Dev)", (context) => {
  let contractProxy: Contract;
  let contracts: HeavyContract[];
  let proxyInterface: Interface;
  let batchInterface: Interface;

  before("Deploy the contracts from range 6000-XXXX", async function () {
    // Deploy the CallForwarder contract
    const creation = await createContract(context, "CallForwarder");
    contractProxy = creation.contract;
    await context.createBlock(creation.rawTx);

    // Get the CallForwarder contract interface
    proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);

    // Deploy heavy contracts (test won't use more than what is needed for reaching max pov)
    contracts = await deployHeavyContracts(
      context,
      6000,
      Number(6000n + MAX_ETH_POV_PER_TX / 24_000n + 1n)
    );

    // Get the interface for Batch precompile
    batchInterface = new ethers.utils.Interface(
      getCompiled("precompiles/batch/Batch").contract.abi
    );
  });

  it("should allow to produce block under the PoV Limit with precompile tx", async function () {
    const max_contracts = MAX_ETH_POV_PER_TX / 24_000n - 1n;

    const { result, block } = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_BATCH_ADDRESS,
        data: batchInterface.encodeFunctionData("batchAll", [
          [contractProxy.options.address],
          [],
          [
            proxyInterface.encodeFunctionData("callRange", [
              contracts[0].account,
              contracts[Number(max_contracts)].account,
            ]),
          ],
          [],
        ]),
        gas: 13_000_000,
      })
    );
    expect(block.proof_size).to.be.at.least(Number(MAX_ETH_POV_PER_TX - 20_000n));
    expect(block.proof_size).to.be.at.most(Number(MAX_ETH_POV_PER_TX - 1n));
    expect(result.successful).to.equal(true);
  });

  it("should prevent a tx reaching just over the PoV with a precompile tx", async function () {
    const max_contracts = MAX_ETH_POV_PER_TX / 24_000n;

    const { result, block } = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_BATCH_ADDRESS,
        data: batchInterface.encodeFunctionData("batchAll", [
          [contractProxy.options.address],
          [],
          [
            proxyInterface.encodeFunctionData("callRange", [
              contracts[0].account,
              contracts[Number(max_contracts)].account,
            ]),
          ],
          [],
        ]),
        gas: 15_000_000,
      })
    );

    // Empty blocks usually do not exceed 10kb, picking 50kb as a safe limit
    expect(block.proof_size).to.be.at.most(50_000);
    expect(result.successful).to.equal(false);
  });
});
