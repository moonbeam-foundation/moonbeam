import {
  prodParasKusama,
  prodParasPolkadot,
  prodParasKusamaCommon,
  prodParasPolkadotCommon,
} from "@polkadot/apps-config/endpoints";

export interface ForeignChainsInfo {
  moonbeamNetworkName: string;
  moonbeamParaId: number;
  foreignChains: ForeignChainInfo[];
}

export interface ForeignChainInfo {
  name: string;
  paraId: number;
  mutedUntil?: number;
  endpoints?: string[];
}

export const getEndpoints = (relay: "Polkadot" | "Kusama" | "Unsupported", paraId: number) => {
  switch (relay) {
    case "Polkadot":
      if (paraId < 2000) {
        const commonGoodPolka = prodParasPolkadotCommon.find((a) => a.paraId === paraId);
        return Object.values(commonGoodPolka.providers);
      }
      const polkaPara = prodParasPolkadot.find((a) => a.paraId === paraId);
      return Object.values(polkaPara.providers);
    case "Kusama":
      if (paraId < 2000) {
        const commonGoodKusama = prodParasKusamaCommon.find((a) => a.paraId === paraId);
        return Object.values(commonGoodKusama.providers);
      }
      const kusamaPara = prodParasKusama.find((a) => a.paraId === paraId);
      return Object.values(kusamaPara.providers);
    case "Unsupported":
      throw new Error("Unsupported chain.");
  }
};

export const isMuted = (moonbeamNetworkName: string, paraId: number) => {
  const info = ForeignChainsEndpoints.find((a) => a.moonbeamNetworkName === moonbeamNetworkName);

  if (info) {
    const match = info.foreignChains.find((a) => a.paraId === paraId);

    if (!match) {
      console.error(`⚠️  No static data for ParaId ${paraId}, please add to foreign-chains.ts`);
      return false;
    }

    const currentTime = new Date().getTime();
    return match.mutedUntil && match.mutedUntil >= currentTime;
  } else return false;
};

export const ForeignChainsEndpoints: ForeignChainsInfo[] = [
  {
    moonbeamNetworkName: "Moonriver",
    moonbeamParaId: 2023,
    foreignChains: [
      {
        name: "Statemine",
        paraId: 1000,
      },
      {
        name: "Karura",
        paraId: 2000,
      },
      {
        name: "Bifrost",
        paraId: 2001,
        mutedUntil: new Date("2023-04-04").getTime(),
      },
      {
        name: "Khala",
        paraId: 2004,
      },
      {
        name: "Shiden",
        paraId: 2007,
      },
      {
        name: "Crust",
        paraId: 2012,
      },
      {
        name: "Integritee",
        paraId: 2015,
      },
      {
        name: "Robonomics",
        paraId: 2048,
      },
      {
        name: "Calamari",
        paraId: 2084,
      },
      {
        name: "Heiko",
        paraId: 2085,
      },
      {
        name: "Kintsugi",
        paraId: 2092,
      },
      {
        name: "Darwinia Crab",
        paraId: 2105,
      },
      {
        name: "Litmus",
        paraId: 2106,
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
      },
      {
        name: "Acala",
        paraId: 2000,
      },
      {
        name: "Astar",
        paraId: 2006,
      },
      {
        name: "Parallel",
        paraId: 2012,
      },
      {
        name: "Bifrost",
        paraId: 2030,
      },
      {
        name: "Centrifuge",
        paraId: 2031,
      },
      {
        name: "Interlay",
        paraId: 2032,
      },
      {
        name: "Phala",
        paraId: 2035,
      },
      {
        name: "Darwinia",
        paraId: 2046,
      },
      {
        name: "Equilibrium",
        paraId: 2011,
      },
    ],
  },
];
