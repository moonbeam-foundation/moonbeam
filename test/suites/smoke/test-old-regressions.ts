import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect, customDevRpcRequest } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { error } from "console";
import { ethers } from "ethers";
import { encodeFunctionData, Hash } from "viem";

// Each case has
// - Contract Address
// - Error started at block
// - Call data

enum Network {
  Moonbeam = "moonbeam",
  Moonriver = "moonriver",
  Moonbase = "moonbase",
}

// Issues/Regressions
class BadBlockRegressionCase {
  issue: string;
  network: Network;
  contractAddress: `0x${string}`;
  block: "latest" | "earliest" | "pending" | "safe" | "finalized" | BigInt;
  callData: `0x${string}`;
}
// MOON-2824
const moon2824: BadBlockRegressionCase = {
  issue: "MOON-2824",
  network: Network.Moonriver,
  contractAddress: "0x1b30a3b5744e733d8d2f19f0812e3f79152a8777",
  block: 1471037n,
  callData: encodeFunctionData({
    abi: [
      {
        inputs: [
          {
            internalType: "address",
            name: "who",
            type: "address",
          },
          {
            internalType: "uint256",
            name: "n",
            type: "uint256",
          },
        ],
        name: "balanceOf",
        outputs: [
          {
            internalType: "uint256",
            name: "",
            type: "uint256",
          },
        ],
        stateMutability: "view",
        type: "function",
      },
    ],
    functionName: "balanceOf",
    args: ["0x30763be2bf075c3fDeA704c5f59A76d011d02943", 2n],
  }),
};

// MOON-2822
const moon2822: BadBlockRegressionCase = {
  issue: "MOON-2822",
  network: Network.Moonbeam,
  contractAddress: "0xFfFFfFff1FcaCBd218EDc0EbA20Fc2308C778080",
  block: "latest",
  callData: encodeFunctionData({
    abi: [
      {
        inputs: [],
        name: "totalSupply",
        outputs: [],
        stateMutability: "nonpayable",
        type: "function",
      },
    ],
    functionName: "totalSupply",
    args: [],
  }),
};

// Group all cases
const cases = [moon2824, moon2822];

describeSuite({
  id: "S15",
  title: "Verify regressions which happened in the past by reading historical state",
  foundationMethods: "read_only",
  testCases: async ({ context, it, log }) => {
    let paraApi: ApiPromise;

    beforeAll(async function () {
      paraApi = context.polkadotJs("para");
      const chainId = (await paraApi.query.ethereumChainId.chainId()).toString();
      log(`Loading test data for chainId ${chainId}.`);
    });

    it({
      id: "C001",
      title: "Verify all bad block regression cases",
      test: async function () {
        for (const testCase of cases) {
          if (testCase.network != (paraApi.consts.system.version.specName.toString() as Network)) {
            log(`Skipping... (Issue ${testCase.issue} specific for ${testCase.network})`);
            continue;
          }

          let callParams = {
            to: testCase.contractAddress,
            data: testCase.callData,
          };
          // Add either blockTag or blockNumber depending on the case specification
          if (typeof testCase.block === "string") {
            callParams["blockTag"] = testCase.block;
          } else if (typeof testCase.block === "bigint") {
            callParams["blockNumber"] = testCase.block;
          }

          const result = await context.viem().call(callParams);

          try {
            expect(result.data).to.contain("0x");
          } catch (e) {
            log(
              `Error found for ${testCase.issue} at block ${testCase.block.toString()}: ${
                result.data
              }`
            );
            throw e;
          }
        }
      },
    });

    it({
      id: "C002",
      title: "Verify bad transaction tracing case",
      chainType: "moonbeam",
      test: async function () {
        const badTxHash = "0xd91d98b539720d8a42069268126d366fd29165e487d94b165a97e0158842657b";
        // Fetch and verify the trace of a bad transaction observed in client version 0.38
        // Detailed in MOON-2702

        const providers = [
          "https://del-moon-rpc-1-moonbeam-rpc-graph-1.moonbeam.ol-infra.network",
          "https://moonbeam.unitedbloc.com",
        ];

        for (const provider of providers) {
          const versionRes = await fetch(provider, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              id: 1,
              jsonrpc: "2.0",
              method: "system_version",
              params: [],
            }),
          });
          const version = await versionRes.json();
          log(
            `Testing for tracing endpoint ${provider} running Moonbeam version: ${version.result}`
          );

          const traceTx = await fetch(provider, {
            method: "POST",
            headers: {
              "Content-Type": "application/json",
            },
            body: JSON.stringify({
              id: 1,
              jsonrpc: "2.0",
              method: "debug_traceTransaction",
              params: [badTxHash, { tracer: "callTracer" }],
            }),
          });

          const traceData = await traceTx.json();

          try {
            expect(traceData.result.from).toBe("0x7369626cee070000000000000000000000000000");
            expect(traceData.result.to).toBe("0xef81930aa8ed07c17948b2e26b7bfaf20144ef2a");
            expect(traceData.result.gas).toBe("0xa6f91");
            expect(traceData.result.gasUsed).toBe("0x8cef");
          } catch (e) {
            error(
              `Error in tracing TX ${badTxHash} using ${provider} running Moonbeam version: ${version.result}`
            );
            throw e;
          }
        }
      },
    });
  },
});
