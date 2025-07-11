import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { createEthersTransaction } from "@moonwall/util";
import { type Abi, encodeFunctionData } from "viem";
import { type HeavyContract, deployHeavyContracts, ConstantStore } from "../../../../helpers";

describeSuite({
  id: "D022702",
  title: "PoV Limit (7.5Mb in Dev)",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let proxyAddress: `0x${string}`;
    let proxyAbi: Abi;
    let contracts: HeavyContract[];
    let callData: `0x${string}`;
    let emptyBlockProofSize: bigint;
    const MAX_CONTRACTS = 20;
    let MAX_ETH_POV_PER_TX: bigint;

    beforeAll(async () => {
      const specVersion = (await context.polkadotJs().runtimeVersion.specVersion).toNumber();
      const constants = ConstantStore(context);
      MAX_ETH_POV_PER_TX = constants.MAX_ETH_POV_PER_TX.get(specVersion);

      // Create an empty block to estimate empty block proof size
      const { block } = await context.createBlock();
      // Empty blocks usually do not exceed 50kb
      emptyBlockProofSize = BigInt(block.proofSize || 50_000);

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
          gasLimit: 52_000_000,
          txnType: "eip1559",
        });

        const { result, block } = await context.createBlock(rawSigned);

        log(`block.proofSize: ${block.proofSize} (successful: ${result?.successful})`);
        expect(block.proofSize).toBeGreaterThanOrEqual(105_000n);
        expect(block.proofSize).toBeLessThanOrEqual(105_000n + emptyBlockProofSize);
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
          gasLimit: 60_000_000,
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
