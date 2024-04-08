import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { Abi, encodeFunctionData } from "viem";
import { expectEVMResult, HeavyContract, deployHeavyContracts } from "../../../../helpers";

describeSuite({
  id: "D012801",
  title: "PoV controlled by gasLimit",
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
      contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);

      callData = encodeFunctionData({
        abi: proxyAbi,
        functionName: "callRange",
        args: [contracts[0].account, contracts[MAX_CONTRACTS].account],
      });
    });

    it({
      id: "T01",
      title: "should allow to include transaction with estimate gas to cover PoV",
      test: async function () {
        const gasEstimate = await context.viem().estimateGas({
          account: ALITH_ADDRESS,
          to: proxyAddress,
          value: 0n,
          data: callData,
        });

        const rawSigned = await createEthersTransaction(context, {
          to: proxyAddress,
          data: callData,
          txnType: "eip1559",
          gasLimit: gasEstimate,
        });

        const { result, block } = await context.createBlock(rawSigned);

        log(`block.proofSize: ${block.proofSize} (successful: ${result?.successful})`);
        expect(block.proofSize).toBeGreaterThanOrEqual(EXPECTED_POV_ROUGH / 1.1);
        expect(block.proofSize).toBeLessThanOrEqual(EXPECTED_POV_ROUGH * 1.1);
        expect(result?.successful).to.equal(true);
      },
    });

    it({
      id: "T02",
      title: "should allow to include transaction with enough gas limit to cover PoV",
      test: async function () {
        const rawSigned = await createEthersTransaction(context, {
          to: proxyAddress,
          data: callData,
          txnType: "eip1559",
          gasLimit: 12_000_000,
        });

        const { result, block } = await context.createBlock(rawSigned);

        log(`block.proof_size: ${block.proofSize} (successful: ${result?.successful})`);
        expect(block.proofSize).to.be.at.least(EXPECTED_POV_ROUGH / 1.1);
        expect(block.proofSize).to.be.at.most(EXPECTED_POV_ROUGH * 1.1);
        expect(result?.successful).to.equal(true);
      },
    });

    it({
      id: "T03",
      title: "should fail to include transaction without enough gas limit to cover PoV",
      test: async function () {
        // This execution uses only < 100k Gas in cpu execute but require 2M Gas for PoV.
        // We are providing only 1M Gas, so it should fail.
        const rawSigned = await createEthersTransaction(context, {
          to: proxyAddress,
          data: callData,
          txnType: "eip1559",
          gasLimit: 1_000_000,
        });

        const { result, block } = await context.createBlock(rawSigned);

        log(`block.proof_size: ${block.proofSize} (successful: ${result?.successful})`);
        // The block still contain the failed (out of gas) transaction so the PoV is still included
        // in the block.
        // 1M Gas allows ~62k of PoV, so we verify we are within range.
        expect(block.proofSize).to.be.at.least(50_000);
        expect(block.proofSize).to.be.at.most(100_000);
        expect(result?.successful).to.equal(true);
        expectEVMResult(result!.events, "Error", "OutOfGas");
      },
    });
  },
});
