import "@moonbeam-network/api-augment";
import {
  describeSuite,
  expect,
  beforeAll,
  deployCreateCompiledContract,
  fetchCompiledContract,
} from "@moonwall/cli";
import { HeavyContract, deployHeavyContracts, expectEVMResult } from "../../../../helpers";

import { Abi, encodeFunctionData } from "viem";
import { ALITH_ADDRESS, PRECOMPILE_BATCH_ADDRESS, createEthersTransaction } from "@moonwall/util";

describeSuite({
  id: "D012704",
  title: "PoV precompile test - gasLimit",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let contracts: HeavyContract[];
    const MAX_CONTRACTS = 50;
    const EXPECTED_POV_ROUGH = 20_000; // bytes
    let batchAbi: Abi;
    let proxyAbi: Abi;
    let proxyAddress: `0x${string}`;
    let callData: `0x${string}`;

    beforeAll(async function () {
      const { contractAddress: contractAdd1, abi } = await deployCreateCompiledContract(
        context,
        "CallForwarder"
      );
      proxyAddress = contractAdd1;
      proxyAbi = abi;
      contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);

      // Get the interface for Batch precompile
      batchAbi = fetchCompiledContract("Batch").abi;

      callData = encodeFunctionData({
        abi: batchAbi,
        functionName: "batchAll",
        args: [
          [proxyAddress],
          [],
          [
            encodeFunctionData({
              abi: proxyAbi,
              functionName: "callRange",
              args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
            }),
          ],
          [],
        ],
      });
    });

    it({
      id: "T01",
      title: "gas cost should have increased with POV",
      test: async function () {
        // Previously this tx cost was ~500K gas -> now it is about 5M due to POV.
        // We pass 1M, so it should fail.
        const rawSigned = await createEthersTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          data: callData,
          gasLimit: 100_000,
          txnType: "eip1559",
        });

        const { result, block } = await context.createBlock(rawSigned);

        // With 1M gas we are allowed to use ~62kb of POV, so verify the range.
        // The tx is still included in the block because it contains the failed tx,
        // so POV is included in the block as well.
        expect(block.proofSize).to.be.at.least(15_000);
        expect(block.proofSize).to.be.at.most(30_000);
        expect(result?.successful).to.equal(true);
        expectEVMResult(result!.events, "Error", "OutOfGas");
      },
    });

    it({
      id: "T02",
      title: "should be able to create a block using the estimated amount of gas",
      test: async function () {
        const gasEstimate = await context.viem().estimateGas({
          account: ALITH_ADDRESS,
          to: PRECOMPILE_BATCH_ADDRESS,
          data: callData,
        });

        const rawSigned = await createEthersTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          data: callData,
          gasLimit: gasEstimate,
          txnType: "eip1559",
        });

        const { result, block } = await context.createBlock(rawSigned);
        expect(block.proofSize).to.be.at.least(EXPECTED_POV_ROUGH / 1.3);
        expect(block.proofSize).to.be.at.most(EXPECTED_POV_ROUGH * 1.3);
        expect(result?.successful).to.equal(true);
        expectEVMResult(result!.events, "Succeed", "Returned");
      },
    });

    it({
      id: "T03",
      title: "should allow to call a precompile tx with enough gas limit to cover PoV",
      test: async function () {
        const rawSigned = await createEthersTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          data: callData,
          gasLimit: 24_000_000,
          txnType: "eip1559",
        });

        const { result, block } = await context.createBlock(rawSigned);
        expect(block.proofSize).to.be.at.least(EXPECTED_POV_ROUGH / 1.3);
        expect(block.proofSize).to.be.at.most(EXPECTED_POV_ROUGH * 1.3);
        expect(result?.successful).to.equal(true);
        expectEVMResult(result!.events, "Succeed", "Returned");
      },
    });
  },
});

// describeDevMoonbeam("PoV precompile test - PoV Limit (3.5Mb in Dev)", (context) => {
//   let contractProxy: Contract;
//   let contracts: HeavyContract[];
//   let proxyInterface: Interface;
//   let batchInterface: Interface;

//   before("Deploy the contracts from range 6000-XXXX", async function () {
//     // Deploy the CallForwarder contract
//     const creation = await createContract(context, "CallForwarder");
//     contractProxy = creation.contract;
//     await context.createBlock(creation.rawTx);

//     // Get the CallForwarder contract interface
//     proxyInterface = new ethers.utils.Interface(getCompiled("CallForwarder").contract.abi);

//     // Deploy heavy contracts (test won't use more than what is needed for reaching max pov)
//     contracts = await deployHeavyContracts(
//       context,
//       6000,
//       Number(6000n + MAX_ETH_POV_PER_TX / 24_000n + 1n)
//     );

//     // Get the interface for Batch precompile
//     batchInterface = new ethers.utils.Interface(
//       getCompiled("precompiles/batch/Batch").contract.abi
//     );
//   });

//   it("should allow to produce block under the PoV Limit with precompile tx", async function () {
//     const max_contracts = MAX_ETH_POV_PER_TX / 24_000n - 1n;

//     const { result, block } = await context.createBlock(
//       createTransaction(context, {
//         to: PRECOMPILE_BATCH_ADDRESS,
//         data: batchInterface.encodeFunctionData("batchAll", [
//           [contractProxy.options.address],
//           [],
//           [
//             proxyInterface.encodeFunctionData("callRange", [
//               contracts[0].account,
//               contracts[Number(max_contracts)].account,
//             ]),
//           ],
//           [],
//         ]),
//         gas: 13_000_000,
//       })
//     );
//     expect(block.proofSize).to.be.at.least(Number(MAX_ETH_POV_PER_TX - 20_000n));
//     expect(block.proofSize).to.be.at.most(Number(MAX_ETH_POV_PER_TX - 1n));
//     expect(result.successful).to.equal(true);
//   });

//   it("should prevent a tx reaching just over the PoV with a precompile tx", async function () {
//     const max_contracts = MAX_ETH_POV_PER_TX / 24_000n;

//     const { result, block } = await context.createBlock(
//       createTransaction(context, {
//         to: PRECOMPILE_BATCH_ADDRESS,
//         data: batchInterface.encodeFunctionData("batchAll", [
//           [contractProxy.options.address],
//           [],
//           [
//             proxyInterface.encodeFunctionData("callRange", [
//               contracts[0].account,
//               contracts[Number(max_contracts)].account,
//             ]),
//           ],
//           [],
//         ]),
//         gas: 15_000_000,
//       })
//     );

//     // Empty blocks usually do not exceed 10kb, picking 50kb as a safe limit
//     expect(block.proofSize).to.be.at.most(50_000);
//     expect(result.successful).to.equal(false);
//   });
// });
