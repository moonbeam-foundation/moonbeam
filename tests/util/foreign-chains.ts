export interface ForeignChainsInfo {
  moonbeamNetworkName: string;
  moonbeamParaId: number;
  foreignChains: ForeignChainInfo[];
}

export interface ForeignChainInfo {
  name: string;
  paraId: number;
  muted?: boolean;
  endpoints: string[];
}

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
        endpoints: ["wss://rpc-shadow.crust.network/"],
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
        muted: true,
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
  {
    moonbeamNetworkName: "Moonbeam",
    moonbeamParaId: 2004,
    foreignChains: [
      {
        name: "Statemint",
        paraId: 1000,
        endpoints: [
          "wss://statemint-rpc.polkadot.io",
          "wss://statemint.api.onfinality.io/public-ws",
          "wss://statemint-rpc.dwellir.com",
          "wss://public-rpc.pinknode.io/statemint",
          "wss://statemint.public.curie.radiumblock.xyz/ws",
        ],
      },
      {
        name: "Acala",
        paraId: 2000,
        endpoints: [
          "wss://acala-rpc-0.aca-api.network",
          "wss://acala-rpc-1.aca-api.network",
          "wss://acala-rpc-2.aca-api.network/ws",
          "wss://acala-rpc-3.aca-api.network/ws",
          "wss://acala.polkawallet.io",
          "wss://acala-polkadot.api.onfinality.io/public-ws",
          "wss://acala-rpc.dwellir.com",
          "wss://1rpc.io/aca",
        ],
      },
      {
        name: "Astar",
        paraId: 2006,
        endpoints: [
          "wss://rpc.astar.network",
          "wss://astar.public.blastapi.io",
          "wss://astar-rpc.dwellir.com",
          "wss://astar.api.onfinality.io/public-ws",
          "wss://astar.public.curie.radiumblock.co/ws",
          "wss://public-rpc.pinknode.io/astar",
          "wss://1rpc.io/astr",
        ],
      },
      {
        name: "Parallel",
        paraId: 2012,
        endpoints: ["wss://rpc.parallel.fi"],
      },
      {
        name: "Bifrost",
        paraId: 2030,
        endpoints: [
          "wss://hk.p.bifrost-rpc.liebi.com/ws",
          "wss://bifrost-polkadot.api.onfinality.io/public-ws",
        ],
      },
      {
        name: "Centrifuge",
        paraId: 2031,
        endpoints: [
          "wss://fullnode.parachain.centrifuge.io",
          "wss://centrifuge-parachain.api.onfinality.io/public-ws",
        ],
      },
      {
        name: "Interlay",
        paraId: 2032,
        endpoints: [
          "wss://api.interlay.io/parachain",
          "wss://interlay.api.onfinality.io/public-ws",
        ],
      },
      {
        name: "Phala",
        paraId: 2035,
        endpoints: ["wss://api.phala.network/ws", "wss://phala.api.onfinality.io/public-ws"],
      },
      {
        name: "Darwinia",
        paraId: 2046,
        endpoints: ["wss://parachain-rpc.darwinia.network"],
      },
    ],
  },
];
