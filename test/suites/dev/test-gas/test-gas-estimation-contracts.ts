import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  PRECOMPILE_BATCH_ADDRESS,
  deployCreateCompiledContract,
  getCompiled,
} from "@moonwall/util";
import { customDevRpcRequest } from "../../../helpers/common.js";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D1703",
  title: "Estimate Gas - Contract estimation",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: `T01`,
      title: `evm should return invalid opcode`,
      test: async function () {
        expect(
          async () =>
            await customDevRpcRequest("eth_estimateGas", [
              {
                from: ALITH_ADDRESS,
                data: "0xe4",
              },
            ])
        ).rejects.toThrowError("evm error: InvalidCode(Opcode(228))");
      },
    });

    it({
      id: "T02",
      title: "eth_estimateGas 0x0 gasPrice is equivalent to not setting one",
      test: async function () {
        const { byteCode } = getCompiled("Incrementor");

        const result = await context.viemClient("public").estimateGas({
          account: ALITH_ADDRESS,
          data: byteCode,
          gasPrice: 0n,
        });
        expect(result).to.equal(174759n);

        const result2 = await context.viemClient("public").estimateGas({
          account: ALITH_ADDRESS,
          data: byteCode,
        });
        expect(result2).to.equal(174759n);
      },
    });

    it({
      id: "T03",
      title: "all batch functions should estimate the same cost",
      test: async function () {
        const { contractAddress: proxyAddress, abi: proxyAbi } = await deployCreateCompiledContract(
          context,
          "CallForwarder"
        );
        const { contractAddress: multiAddress, abi: multiAbi } = await deployCreateCompiledContract(
          context,
          "MultiplyBy7"
        );
        const batchAbi = getCompiled("precompiles/batch/Batch").contract.abi;

        const callParameters = [
          [proxyAddress, proxyAddress],
          [],
          [
            encodeFunctionData({
              abi: proxyAbi,
              functionName: "call",
              args: [
                multiAddress,
                encodeFunctionData({
                  abi: multiAbi,
                  functionName: "multiply",
                  args: [42],
                }),
              ],
            }),
            encodeFunctionData({
              abi: proxyAbi,
              functionName: "delegateCall",
              args: [
                multiAddress,
                encodeFunctionData({
                  abi: multiAbi,
                  functionName: "multiply",
                  args: [42],
                }),
              ],
            }),
          ],
          [],
        ];

        const batchSomeGas = await context.viemClient("public").estimateGas({
          account: ALITH_ADDRESS,
          to: PRECOMPILE_BATCH_ADDRESS,
          data: encodeFunctionData({
            abi: batchAbi,
            functionName: "batchSome",
            args: callParameters,
          }),
        });

        const batchSomeUntilFailureGas = await context.viemClient("public").estimateGas({
          account: ALITH_ADDRESS,
          to: PRECOMPILE_BATCH_ADDRESS,
          data: encodeFunctionData({
            abi: batchAbi,
            functionName: "batchSomeUntilFailure",
            args: callParameters,
          }),
        });

        const batchAllGas = await context.viemClient("public").estimateGas({
          account: ALITH_ADDRESS,
          to: PRECOMPILE_BATCH_ADDRESS,
          data: encodeFunctionData({
            abi: batchAbi,
            functionName: "batchAll",
            args: callParameters,
          }),
        });

        expect(batchSomeGas).to.be.eq(batchAllGas);
        expect(batchSomeUntilFailureGas).to.be.eq(batchAllGas);
      },
    });

    it({
      id: "T04",
      title: "Non-transactional calls allowed from e.g. precompile address",
      test: async function () {
        const { byteCode } = getCompiled("MultiplyBy7");
        expect(
          await context.viemClient("public").estimateGas({
            account: PRECOMPILE_BATCH_ADDRESS,
            data: byteCode,
          })
        ).toBe(156994n);
      },
    });

    it({
      id: "T05",
      title: "Should be able to estimate gas of infinite loop call",
      test: async function () {
        const { contractAddress, abi } = await deployCreateCompiledContract(context, "Looper");

        expect(
          async () =>
            await context.viemClient("public").estimateGas({
              account: ALITH_ADDRESS,
              to: contractAddress,
              data: encodeFunctionData({
                abi: abi,
                functionName: "infinite",
                args: [],
              }),
            })
        ).rejects.toThrowError("gas required exceeds allowance 1500000");
      },
    });
  },
});
