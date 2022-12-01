export interface ForeignChainsInfo {
  moonbeamNetworkName: string;
  moonbeamParaId: number;
  foreignChains: ForeignChainInfo[];
}

export interface ForeignChainInfo {
  name: string;
  paraId: number;
  endpoints: string[];
}

export const mutedChains = [2092]

export const ForeignChainsEndpoints: ForeignChainsInfo[] = [
  {
    moonbeamNetworkName: "Moonriver",
    moonbeamParaId: 2023,
    foreignChains: [
      {
        name: "Statemine",
        paraId: 1000,
        endpoints: [
          "wss://statemine-rpc.polkadot.io",
          "wss://statemine.api.onfinality.io/public-ws",
          "wss://statemine-rpc.dwellir.com",
          "wss://public-rpc.pinknode.io/statemine",
          "wss://statemine.public.curie.radiumblock.co/ws",
        ],
      },
      {
        name: "Karura",
        paraId: 2000,
        endpoints: [
          "wss://karura-rpc-0.aca-api.network",
          "wss://karura-rpc-1.aca-api.network",
          "wss://karura-rpc-2.aca-api.network/ws",
          "wss://karura-rpc-3.aca-api.network/ws",
        ],
      },
      {
        name: "Bifrost",
        paraId: 2001,
        endpoints: [
          "wss://bifrost-rpc.liebi.com/ws",
          "wss://bifrost-parachain.api.onfinality.io/public-ws",
          "wss://bifrost-rpc.dwellir.com",
        ],
      },
      {
        name: "Khala",
        paraId: 2004,
        endpoints: [
          "wss://khala-api.phala.network/ws",
          "wss://khala.api.onfinality.io/public-ws",
          "wss://khala-rpc.dwellir.com",
          "wss://public-rpc.pinknode.io/khala",
        ],
      },
      {
        name: "Shiden",
        paraId: 2007,
        endpoints: [
          "wss://rpc.shiden.astar.network",
          "wss://shiden.public.blastapi.io",
          "wss://shiden-rpc.dwellir.com",
          "wss://shiden.api.onfinality.io/public-ws",
          "wss://public-rpc.pinknode.io/shiden",
        ],
      },
      {
        name: "Crust",
        paraId: 2012,
        endpoints: ["wss://rpc-shadow.crust.network/x"],
      },
      {
        name: "Integritee",
        paraId: 2015,
        endpoints: [
          "wss://kusama.api.integritee.network",
          "wss://integritee-kusama.api.onfinality.io/public-ws",
        ],
      },
      {
        name: "Robonomics",
        paraId: 2048,
        endpoints: [
          "wss://kusama.rpc.robonomics.network/",
          "wss://robonomics.api.onfinality.io/public-ws",
          "wss://robonomics.0xsamsara.com",
          "wss://robonomics.leemo.me",
        ],
      },
      {
        name: "Calamari",
        paraId: 2084,
        endpoints: ["wss://ws.calamari.systems/"],
      },
      {
        name: "Heiko",
        paraId: 2085,
        endpoints: ["wss://heiko-rpc.parallel.fi"],
      },
      {
        name: "Kintsugi",
        paraId: 2092,
        endpoints: [
          "wss://api-kusama.interlay.io/parachain",
          "wss://kintsugi.api.onfinality.io/public-ws",
        ],
      },
      {
        name: "Darwinia Crab",
        paraId: 2105,
        endpoints: ["wss://crab-parachain-rpc.darwinia.network/"],
      },
      {
        name: "Litmus",
        paraId: 2106,
        endpoints: ["wss://rpc.litmus-parachain.litentry.io"],
      },
    ],
  },
];
