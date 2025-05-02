import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect } from "@moonwall/cli";
import { encodeFunctionData } from "viem";
import { deployedContractsInLatestBlock } from "../../../../helpers";

describeSuite({
  id: "D015001",
  title: "Precompiles - Validate PrecompileChecks",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let multiplyBy7Contract;

    beforeAll(async () => {
      // Test setup
      multiplyBy7Contract = await context.deployContract!("MultiplyBy7");

      expect(await deployedContractsInLatestBlock(context)).contains(
        multiplyBy7Contract.contractAddress
      );
    });

    it({
      id: "T01",
      title: `Validate "CallableByContract" by calling precompile from smart-contract constructor`,
      test: async function () {
        const contract = await context.deployContract!("CallBatchPrecompileFromConstructor", {
          gas: 5_000_000n,
          rawTxOnly: true,
          args: [
            multiplyBy7Contract.contractAddress,
            [
              encodeFunctionData({
                abi: multiplyBy7Contract.abi,
                functionName: "multiply",
                args: [5],
              }),
            ],
          ],
        });

        const ethEvent = (await context.polkadotJs().query.system.events()).find(({ event }) =>
          context.polkadotJs().events.ethereum.Executed.is(event)
        );
        expect((ethEvent.toHuman() as any).event["data"]["exitReason"]["Revert"]).equals(
          "Reverted"
        );
      },
    });

    it({
      id: "T02",
      title: `Validate "CallableByContract" by calling precompile from smart-contract constructor originated by a subcall`,
      test: async function () {
        const contract = await context.deployContract!(
          "CallBatchPrecompileFromConstructorInSubCall",
          {
            gas: 5_000_000n,
            rawTxOnly: true,
          }
        );
        expect(await deployedContractsInLatestBlock(context)).contains(contract.contractAddress);

        const rawTx = await context.writeContract({
          contractName: "CallBatchPrecompileFromConstructorInSubCall",
          contractAddress: contract.contractAddress,
          functionName: "simple",
          args: [
            multiplyBy7Contract.contractAddress,
            [
              encodeFunctionData({
                abi: multiplyBy7Contract.abi,
                functionName: "multiply",
                args: [5],
              }),
            ],
          ],
          gas: 5_000_000n,
        });
        await context.createBlock(rawTx, { allowFailures: false });

        const ethEvent2 = (await context.polkadotJs().query.system.events()).find(({ event }) =>
          context.polkadotJs().events.ethereum.Executed.is(event)
        );
        expect((ethEvent2.toHuman() as any).event["data"]["exitReason"]["Revert"]).equals(
          "Reverted"
        );
      },
    });
  },
});
