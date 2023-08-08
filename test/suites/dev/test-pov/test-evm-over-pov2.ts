import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { MAX_ETH_POV_PER_TX, createEthersTransaction } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import { HeavyContract, deployHeavyContracts } from "../../../helpers/contracts.js";

describeSuite({
  id: "D2402",
  title: "PoV Limit (3.5Mb in Dev)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let proxyAddress: `0x${string}`;
    let proxyAbi: Abi;
    let contracts: HeavyContract[];
    let callData: `0x${string}`;
    const MAX_CONTRACTS = 20;
    const EXPECTED_POV_ROUGH = 500_000; // bytes

    beforeAll(async () => {
      const { contractAddress, abi } = await deployCreateCompiledContract(context, "CallForwarder");
      proxyAddress = contractAddress;
      proxyAbi = abi;

      // Deploy heavy contracts (test won't use more than what is needed for reaching max pov)
      contracts = await deployHeavyContracts(
        context,
        6000,
        Number(6000n + MAX_ETH_POV_PER_TX / 24_000n + 1n)
      );

      callData = encodeFunctionData({
        abi: proxyAbi,
        functionName: "callRange",
        args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
      });
    });

    it({
      id: "T01",
      title: "should allow to produce block just under the PoV Limit",
      test: async function () {
        const calculatedMax = MAX_ETH_POV_PER_TX / 24_000n - 1n;

        const callData = encodeFunctionData({
          abi: proxyAbi,
          functionName: "callRange",
          args: [contracts[0].account, contracts[Number(calculatedMax)].account],
        });

        const rawSigned = await createEthersTransaction(context, {
          to: proxyAddress,
          data: callData,
          gasLimit: 13_000_000,
          txnType: "eip1559",
        });

        const { result, block } = await context.createBlock(rawSigned);

        log(`block.proofSize: ${block.proofSize} (successful: ${result?.successful})`);
        expect(block.proofSize).toBeGreaterThanOrEqual(MAX_ETH_POV_PER_TX - 20_000n);
        expect(block.proofSize).toBeLessThanOrEqual(MAX_ETH_POV_PER_TX - 1n);
        expect(result?.successful).to.equal(true);
      },
    });

    it({
      id: "T02",
      title: "should prevent a transaction reaching just over the PoV",
      test: async function () {
        const calculatedMax = MAX_ETH_POV_PER_TX / 24_000n;

        const callData = encodeFunctionData({
          abi: proxyAbi,
          functionName: "callRange",
          args: [contracts[0].account, contracts[Number(calculatedMax) + 1].account],
        });

        const rawSigned = await createEthersTransaction(context, {
          to: proxyAddress,
          data: callData,
          gasLimit: 15_000_000,
          txnType: "eip1559",
        });

        const { result, block } = await context.createBlock(rawSigned);

        log(`block.proofSize: ${block.proofSize} (successful: ${result?.successful})`);
        // Empty blocks usually do not exceed 10kb, picking 50kb as a safe limit
        expect(block.proofSize).to.be.at.most(50_000);
        expect(result?.successful).to.equal(false);
      },
    });
  },
});
