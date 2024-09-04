import { runtimes } from "helpers/runtimes";
import { Chain, createPublicClient, http } from "viem";
import fs from "fs";
import path from "path";

interface Network {
  name: string;
  endpoint: string;
  id: number;
  currency: string;
}

interface Sample {
  network: string;
  runtime: number;
  blockNumber: number;
  txHash: string;
}

const networks: Network[] = [
  {
    name: "moonbeam",
    endpoint: "https://deo-moon-rpc-1-moonbeam-rpc-graph-1.moonbeam.ol-infra.network",
    id: 1284,
    currency: "GMLR",
  },
  {
    name: "moonriver",
    endpoint: "https://deo-moon-rpc-1-moonriver-rpc-graph-1.moonriver.ol-infra.network",
    id: 1285,
    currency: "MOVR",
  },
  {
    name: "moonbase",
    endpoint: "https://deo-moon-rpc-1-moonbase-alpha-rpc-graph-1.moonbase.ol-infra.network",
    id: 1287,
    currency: "DEV",
  },
];

function createChain(network: Network) {
  const customChain: Chain = {
    name: network.name,
    id: network.id,
    nativeCurrency: {
      name: network.currency,
      symbol: network.currency,
      decimals: 18,
    },
    rpcUrls: {
      default: {
        http: [network.endpoint],
      },
    },
  };
  return customChain;
}

const main = async () => {
  console.log(
    `Generating tracing samples for networks ${networks.flatMap((n) => n.name).join(", ")}`
  );
  networks.forEach(async (network) => {
    let samples: Sample[] = [];
    const chain = createChain(network);
    const client = createPublicClient({
      chain,
      transport: http(network.endpoint),
    });
    const relevantRuntimes = runtimes
      .filter((r) => r.specVersion >= 400) // Runtimes before 400 don't support tracing
      .filter((r) => r.blockNumber[network.name as keyof typeof r.blockNumber] !== null);

    for (const runtime of relevantRuntimes) {
      console.log(
        `Runtime ${runtime.specVersion} has block number ${
          runtime.blockNumber[network.name as keyof typeof runtime.blockNumber]
        }`
      );
      let block = await client.getBlock({
        blockNumber: runtime.blockNumber[network.name as keyof typeof runtime.blockNumber]!,
      });
      let blocksAhead = 0n;
      while (block.transactions.length === 0) {
        blocksAhead += 1n;
        block = await client.getBlock({
          blockNumber: block.number + blocksAhead,
        });
      }
      console.log(
        `Runtime ${network.name}-${runtime.specVersion}: Block ${
          block.number + blocksAhead
        } found ${blocksAhead} blocks ahead has ${block.transactions.length} transactions`
      );

      // Create a sample
      const sample: Sample = {
        network: network.name,
        runtime: runtime.specVersion,
        blockNumber: parseInt(block.number.toString()),
        txHash: block.transactions[0],
      };
      samples.push(sample);
    }
    // Save samples as JSON
    const json = JSON.stringify(samples, null, 2);
    const filename = path.join("helpers", `${network.name}-tracing-samples.json`);
    fs.writeFileSync(filename, json);
    console.log(`✅ Saved ${samples.length} samples to ${filename}`);
  });
};

// run
main();
