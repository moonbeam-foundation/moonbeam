import "@moonbeam-network/api-augment";
import Debug from "debug";

import { expect, use as chaiUse } from "chai";
import { ethers } from "ethers";
import chaiAsPromised from "chai-as-promised";

import { alith, baltathar, charleth, ALITH_PRIVATE_KEY } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { createContract, createTransaction } from "../../util/transactions";
import { PRECOMPILE_BATCH_ADDRESS, MAX_ETH_POV_PER_TX } from "../../util/constants";
import { Contract } from "web3-eth-contract";
import { expectEVMResult } from "../../util/eth-transactions";
import { deployHeavyContracts, HeavyContract } from "./test-evm-over-pov";
import { customWeb3Request } from "../../util/providers";
const debug = Debug("test:precompile-over-pov");

chaiUse(chaiAsPromised);

describeDevMoonbeam("PoV precompile test", (context) => {
  let contractProxy: Contract;
  let contracts: HeavyContract[];
  const MAX_CONTRACTS = 50;
  const EXPECTED_POV_ROUGH = 1_000_000; // bytes
  let batchInterface: any;
  let proxyInterface: any;

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
