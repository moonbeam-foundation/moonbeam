import "@moonbeam-network/api-augment";
import {
  beforeAll,
  customDevRpcRequest,
  deployCreateCompiledContract,
  describeSuite,
  expect,
} from "@moonwall/cli";
import { ALITH_ADDRESS, createEthersTransaction } from "@moonwall/util";
import { Abi, decodeEventLog, encodeFunctionData } from "viem";
import { HeavyContract, deployHeavyContracts } from "../../../../helpers";

describeSuite({
  id: "D011805",
  title: "Estimate Gas - subCall",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let callForwarderAddress: `0x${string}`;
    let looperAddress: `0x${string}`;
    let subCallOogAbi: Abi;
    let subCallOogAddress: `0x${string}`;

    let heavyContracts: string[] = [];
    const MAX_HEAVY_CONTRACTS = 15;

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

      // Deploy heavy contracts (test won't use more than what is needed for reaching max pov)
      // heavyContracts = await deployHeavyContracts(context, 6000, 6000 + MAX_HEAVY_CONTRACTS);
      for (let i = 0; i <= MAX_HEAVY_CONTRACTS; i++) {
        const { contractAddress } = await deployCreateCompiledContract(context, "BloatedContract");
        heavyContracts.push(contractAddress);
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
          args: [heavyContracts],
          value: 0n,
        });

        log(`Estimated gas: ${estimatedGas}`);

        const txHash = await context.viem().sendTransaction({
          to: subCallOogAddress,
          data: encodeFunctionData({
            abi: subCallOogAbi,
            functionName: "subCallForwarder",
            args: [heavyContracts],
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
