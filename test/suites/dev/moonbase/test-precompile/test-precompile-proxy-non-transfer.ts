import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  CHARLETH_SESSION_ADDRESS,
  CONTRACT_PROXY_TYPE_NON_TRANSFER,
  DOROTHY_ADDRESS,
  DOROTHY_PRIVATE_KEY,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  PRECOMPILE_PROXY_ADDRESS,
  createViemTransaction,
} from "@moonwall/util";
import { encodeFunctionData } from "viem";
import { expectEVMResult, getAuthorMappingInfo } from "../../../../helpers";

describeSuite({
  id: "D012965",
  title: "Proxy : Non transfer - Evm transfer",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    beforeAll(async () => {
      const rawtxn = await context.writePrecompile!({
        precompileName: "Proxy",
        functionName: "addProxy",
        args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_NON_TRANSFER, 0],
        rawTxOnly: true,
      });
      const { result } = await context.createBlock(rawtxn);
      expectEVMResult(result!.events, "Succeed");
    });

    it({
      id: "T01",
      title: "should fail in simple evm transfer",
      test: async function () {
        const beforeCharlethBalance = await context
          .viem()
          .getBalance({ address: CHARLETH_ADDRESS });

        const { abi } = fetchCompiledContract("Proxy");
        const rawTxn = await createViemTransaction(context, {
          to: PRECOMPILE_PROXY_ADDRESS,
          value: 100n,
          data: encodeFunctionData({
            abi,
            functionName: "proxy",
            args: [ALITH_ADDRESS, CHARLETH_ADDRESS, "0x00"],
          }),
          privateKey: BALTATHAR_PRIVATE_KEY,
          skipEstimation: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");
        expect(
          async () =>
            await createViemTransaction(context, {
              to: PRECOMPILE_PROXY_ADDRESS,
              value: 100n,
              privateKey: BALTATHAR_PRIVATE_KEY,
              data: encodeFunctionData({
                abi,
                functionName: "proxy",
                args: [ALITH_ADDRESS, CHARLETH_ADDRESS, "0x00"],
              }),
            })
        ).rejects.toThrow("CallFiltered");

        const afterCharlethBalance = await context.viem().getBalance({ address: CHARLETH_ADDRESS });
        expect(afterCharlethBalance).toBe(beforeCharlethBalance);
      },
    });

    it({
      id: "T02",
      title: "should succeed in adding an association",
      test: async function () {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [CHARLETH_ADDRESS, CONTRACT_PROXY_TYPE_NON_TRANSFER, 0],
          rawTxOnly: true,
          privateKey: DOROTHY_PRIVATE_KEY,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const { abi } = fetchCompiledContract("AuthorMapping");

        const rawTxn2 = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "proxy",
          rawTxOnly: true,
          privateKey: CHARLETH_PRIVATE_KEY,
          args: [
            DOROTHY_ADDRESS,
            PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
            encodeFunctionData({
              abi,
              functionName: "addAssociation",
              args: [CHARLETH_SESSION_ADDRESS],
            }),
          ],
        });
        const { result: result2 } = await context.createBlock(rawTxn2);
        expectEVMResult(result2!.events, "Succeed");
        expect((await getAuthorMappingInfo(context, CHARLETH_SESSION_ADDRESS))!.account).toBe(
          DOROTHY_ADDRESS
        );
      },
    });
  },
});
