import "@moonbeam-network/api-augment";
import {
  beforeAll,
  deployCreateCompiledContract,
  describeSuite,
  expect,
  fetchCompiledContract,
} from "@moonwall/cli";
import {
  MAX_ETH_POV_PER_TX,
  PRECOMPILE_BATCH_ADDRESS,
  createEthersTransaction,
} from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import { HeavyContract, deployHeavyContracts } from "../../../helpers";

describeSuite({
  id: "D2404",
  title: "PoV precompile test - PoV Limit (3.5Mb in Dev)",
  foundationMethods: "dev",
  testCases: ({ context, log, it }) => {
    let contracts: HeavyContract[];
    let batchAbi: Abi;
    let proxyAbi: Abi;
    let proxyAddress: `0x${string}`;

    beforeAll(async function () {
      const { contractAddress: contractAdd1, abi } = await deployCreateCompiledContract(
        context,
        "CallForwarder"
      );
      proxyAddress = contractAdd1;
      proxyAbi = abi;
      contracts = await deployHeavyContracts(
        context,
        6000,
        Number(6000n + MAX_ETH_POV_PER_TX / 24_000n + 1n)
      );

      // Get the interface for Batch precompile
      batchAbi = fetchCompiledContract("Batch").abi;
    });

    it({
      id: "T01",
      title: "should allow to produce block under the PoV Limit with precompile tx",
      test: async function () {
        const maxContracts = MAX_ETH_POV_PER_TX / 24_000n - 1n;

        const callData = encodeFunctionData({
          abi: batchAbi,
          functionName: "batchAll",
          args: [
            [proxyAddress],
            [],
            [
              encodeFunctionData({
                abi: proxyAbi,
                functionName: "callRange",
                args: [contracts[0].account, contracts[Number(maxContracts)].account],
              }),
            ],
            [],
          ],
        });

        const rawSigned = await createEthersTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          data: callData,
          gasLimit: 13_000_000,
        });

        const { result, block } = await context.createBlock(rawSigned);
        expect(block.proofSize).to.be.at.least(Number(MAX_ETH_POV_PER_TX - 20_000n));
        expect(block.proofSize).to.be.at.most(Number(MAX_ETH_POV_PER_TX - 1n));
        expect(result?.successful).to.equal(true);
      },
    });

    it({
      id: "T0",
      title: "should prevent a tx reaching just over the PoV with a precompile tx",
      test: async function () {
        const maxContracts = MAX_ETH_POV_PER_TX / 24_000n;

        const callData = encodeFunctionData({
          abi: batchAbi,
          functionName: "batchAll",
          args: [
            [proxyAddress],
            [],
            [
              encodeFunctionData({
                abi: proxyAbi,
                functionName: "callRange",
                args: [contracts[0].account, contracts[Number(maxContracts)].account],
              }),
            ],
            [],
          ],
        });

        const rawSigned = await createEthersTransaction(context, {
          to: PRECOMPILE_BATCH_ADDRESS,
          data: callData,
          gasLimit: 15_000_000,
        });

        const { result, block } = await context.createBlock(rawSigned);

        // Empty blocks usually do not exceed 10kb, picking 50kb as a safe limit
        expect(block.proofSize).to.be.at.most(50_000);
        expect(result?.successful).to.equal(false);
      },
    });
  },
});
