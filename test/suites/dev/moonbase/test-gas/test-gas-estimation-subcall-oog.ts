import "@moonbeam-network/api-augment";
import { beforeAll, deployCreateCompiledContract, describeSuite, expect } from "@moonwall/cli";
import { ALITH_ADDRESS } from "@moonwall/util";
import { Abi, decodeEventLog, encodeFunctionData } from "viem";

describeSuite({
  id: "D011805",
  title: "Estimate Gas - subCall",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let callForwarderAddress: `0x${string}`;
    let looperAddress: `0x${string}`;
    let subCallOogAbi: Abi;
    let subCallOogAddress: `0x${string}`;

    let bloatedContracts: string[] = [];
    const MAX_BLOATED_CONTRACTS = 15;

    beforeAll(async function () {
      const { contractAddress } = await deployCreateCompiledContract(context, "CallForwarder");
      callForwarderAddress = contractAddress;

      const { contractAddress: contractAddress2 } = await deployCreateCompiledContract(
        context,
        "Looper"
      );
      looperAddress = contractAddress2;

      const { abi, contractAddress: contractAddress3 } = await deployCreateCompiledContract(
        context,
        "SubCallOOG"
      );
      subCallOogAbi = abi;
      subCallOogAddress = contractAddress3;

      // Deploy bloated contracts (test won't use more than what is needed for reaching max pov)
      for (let i = 0; i <= MAX_BLOATED_CONTRACTS; i++) {
        const { contractAddress } = await deployCreateCompiledContract(context, "BloatedContract");
        bloatedContracts.push(contractAddress);
      }
    });

    it({
      id: "T01",
      title: "gas estimation should make subcall OOG",
      test: async function () {
        const estimatedGas = await context.viem().estimateContractGas({
          account: ALITH_ADDRESS,
          abi: subCallOogAbi,
          address: subCallOogAddress,
          functionName: "subCallLooper",
          maxPriorityFeePerGas: 0n,
          args: [looperAddress, 999],
          value: 0n,
        });

        const txHash = await context.viem().sendTransaction({
          to: subCallOogAddress,
          data: encodeFunctionData({
            abi: subCallOogAbi,
            functionName: "subCallLooper",
            args: [looperAddress, 999],
          }),
          txnType: "eip1559",
          gasLimit: estimatedGas,
        });

        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash: txHash });

        const decoded = decodeEventLog({
          abi: subCallOogAbi,
          data: receipt.logs[0].data,
          topics: receipt.logs[0].topics,
        }) as any;

        expect(decoded.eventName).to.equal("SubCallFail");
      },
    });

    it({
      id: "T02",
      title: "gas estimation should make pov-consuming subcall suceed",
      test: async function () {
        const estimatedGas = await context.viem().estimateContractGas({
          account: ALITH_ADDRESS,
          abi: subCallOogAbi,
          address: subCallOogAddress,
          functionName: "subCallForwarder",
          maxPriorityFeePerGas: 0n,
          args: [bloatedContracts],
          value: 0n,
        });

        log(`Estimated gas: ${estimatedGas}`);

        const txHash = await context.viem().sendTransaction({
          to: subCallOogAddress,
          data: encodeFunctionData({
            abi: subCallOogAbi,
            functionName: "subCallForwarder",
            args: [bloatedContracts],
          }),
          txnType: "eip1559",
          gasLimit: estimatedGas,
        });

        await context.createBlock();

        const receipt = await context.viem().getTransactionReceipt({ hash: txHash });
        const decoded = decodeEventLog({
          abi: subCallOogAbi,
          data: receipt.logs[0].data,
          topics: receipt.logs[0].topics,
        }) as any;
        expect(decoded.eventName).to.equal("SubCallSucceed");
      },
    });
  },
});
