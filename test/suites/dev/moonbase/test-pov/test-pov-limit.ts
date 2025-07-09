import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { type Abi, encodeFunctionData } from "viem";
import { deployHeavyContracts, type HeavyContract } from "../../../../helpers";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";

describeSuite({
  id: "D0121111",
  title: "PoV Limit",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {

    let contractCallForwarderAddress: `0x${string}`;
    let contracts: HeavyContract[];
    let callData: `0x${string}`;
    let proxyAbi: Abi;

    // The goal of this test is to fill the max PoV limit with normal transactions (75% of 10MB)
    const MAX_POV_LIMIT = 10 * 1024 * 1024 * 0.75;
    const MAX_CONTRACTS = Math.floor(MAX_POV_LIMIT / 24_000);

    beforeAll(async function () {
      const { contractAddress: _contractAddress, abi } = await deployCreateCompiledContract(context, "CallForwarder");
      contractCallForwarderAddress = _contractAddress;
      proxyAbi = abi;

      // Deploy heavy contracts (test won't use more than what is needed for reaching max pov)
      contracts = await deployHeavyContracts(context, 6000, 6000 + MAX_CONTRACTS);

      await context.createBlock();
    });

    it({
      id: "T01",
      title: "Should be able to fill 75% of block proof size limit with normal transactions",
      test: async function () {
        const HEAVY_CONTRACTS_PER_TX = 40;
        const transactions: `0x${string}`[] = [];

        // Get initial nonce for Alith
        let nonce = await context.viem().getTransactionCount({address: ALITH_ADDRESS});

        // Split into batches of HEAVY_CONTRACTS_PER_TX contracts
        for (let i = 0; i < MAX_CONTRACTS; i += HEAVY_CONTRACTS_PER_TX) {
          const endIndex = Math.min(i + HEAVY_CONTRACTS_PER_TX, MAX_CONTRACTS);
          
          const callData = encodeFunctionData({
            abi: proxyAbi,
            functionName: "callRange", 
            args: [contracts[i].account, contracts[endIndex].account],
          });

          const gasEstimate = await context.viem().estimateGas({
            account: ALITH_ADDRESS,
            to: contractCallForwarderAddress,
            value: 0n,
            data: callData,
          });

          const rawSigned = await createEthersTransaction(context, {
            to: contractCallForwarderAddress,
            data: callData,
            txnType: "eip1559",
            gasLimit: gasEstimate,
            nonce: nonce++
          });

          transactions.push(rawSigned);
        }

        const { result, block } = await context.createBlock(transactions);

        const blockWeight = await context.polkadotJs().query.system.blockWeight();

        const proofSize =
          blockWeight.normal.proofSize.toBigInt() +
          blockWeight.operational.proofSize.toBigInt() +
          blockWeight.mandatory.proofSize.toBigInt();

        log(`Consumed block proofSize: ${proofSize} / ${MAX_POV_LIMIT}`);
        // In practice the total block proof size consumed should be greather than 75%
        // because some proof size is consumed outisde of normal transactions (e.g. on_initialize stuff)
        expect(
          proofSize,
          `Proof size is not greater than ${MAX_POV_LIMIT}`
        ).toBeGreaterThanOrEqual(MAX_POV_LIMIT);
      },
    });
  },
});
