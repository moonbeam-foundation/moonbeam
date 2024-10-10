import "@moonbeam-network/api-augment";
import { describeSuite, beforeAll, expect } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { encodeFunctionData, Hash } from "viem";
import moonbaseSamples from "../../helpers/moonbase-tracing-samples.json";
import moonbeamSamples from "../../helpers/moonbeam-tracing-samples.json";
import moonriverSamples from "../../helpers/moonriver-tracing-samples.json";
import { generateKeyringPair } from "@moonwall/util";

interface Sample {
  network: string;
  runtime: string;
  blockNumber: number;
  txHash: `0x${string}`;
}

const samples = {
  moonbase: moonbaseSamples,
  moonbeam: moonbeamSamples,
  moonriver: moonriverSamples,
};

enum Network {
  Moonbeam = "moonbeam",
  Moonriver = "moonriver",
  Moonbase = "moonbase",
}

class BadBlockRegressionCase {
  issue: string;
  network: Network;
  contractAddress: `0x${string}`;
  block: "latest" | "earliest" | "pending" | "safe" | "finalized" | bigint;
  callData: `0x${string}`;
}

// Issues/Regressions
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
          let chain = (await paraApi.rpc.system.chain()).toString().toLowerCase();
          if (testCase.network.toString() !== chain) {
            log(`Skipping... (Issue ${testCase.issue} specific for ${testCase.network})`);
            continue;
          }

          const callParams = {
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
            log(`${testCase.issue}: error at block ${testCase.block.toString()}: ${result.data}`);
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
        // Fetch and verify the trace of a bad transaction observed in client version 0.38
        // Detailed in MOON-2702
        const badTxHash = "0xd91d98b539720d8a42069268126d366fd29165e487d94b165a97e0158842657b";

        const traceData = await context.viem().request<TraceTransactionSchema>({
          method: "debug_traceTransaction",
          params: [badTxHash, { tracer: "callTracer" }],
        });

        try {
          expect(traceData.from).toBe("0x7369626cee070000000000000000000000000000");
          expect(traceData.to).toBe("0xef81930aa8ed07c17948b2e26b7bfaf20144ef2a");
          expect(traceData.gas).toBe("0xa6f91");
          expect(traceData.gasUsed).toBe("0x8cef");
        } catch (e) {
          const provider = await context.viem().request<SystemVersionSchema>({
            method: "system_version",
            params: [],
          });
          const url = context.viem().chain.rpcUrls.default;
          log(`Testing for tracing endpoint ${url} running Moonbeam version: ${provider}`);
          throw e;
        }
      },
    });

    it({
      id: "C003",
      title: "Verify tracing works for transactions generated by all runtime versions",
      test: async function () {
        const network = paraApi.consts.system.version.specName.toString() as Network;
        const testSamples = network in samples ? samples[network] : [];
        testSamples.forEach(async (sample) => {
          log(`Testing sample: ${JSON.stringify(sample.txHash)}`);
          const traceData = await context.viem().request<TraceTransactionSchema>({
            method: "debug_traceTransaction",
            params: [sample.txHash as `0x${string}`, { tracer: "callTracer" }],
          });

          try {
            log(`Verifying trace data for sample: ${traceData.from} -> ${traceData.to}`);
            expect(traceData.from).toContain("0x");
            expect(traceData.to).toContain("0x");
          } catch (e) {
            log(`Error found for sample: ${JSON.stringify(sample)}`);
            throw e;
          }
        });
      },
    });

    it({
      id: "C004",
      title: "Moonriver: eth_getLogs with more than 16 addresses filtered should return logs",
      chainType: "moonriver",
      test: async function () {
        const addresses = Array.from({ length: 1024 }, () => generateKeyringPair()).map(
          (a) => a.address as `0x${string}`
        );

        // Original case identified at this particular block height
        const logs = await context.viem().getLogs({
          fromBlock: 7970232n,
          toBlock: 7970232n,
          address: addresses,
        });
        log(`Logs found: ${logs.length}`);
      },
    });
  },
});

type TraceTransactionSchema = {
  Parameters: [
    hash: Hash,
    options:
      | {
          disableStorage?: boolean;
          disableStack?: boolean;
          enableMemory?: boolean;
          enableReturnData?: boolean;
          tracer?: string;
        }
      | {
          timeout?: string;
          tracerConfig?: {
            onlyTopCall?: boolean;
            withLog?: boolean;
          };
        }
      | undefined
  ];
  ReturnType: {
    from: string;
    to: string;
    gas: string;
    gasUsed: string;
  };
};

type SystemVersionSchema = {
  Parameters: [];
  ReturnType: any;
};
