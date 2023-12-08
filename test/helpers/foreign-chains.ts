import {
  prodParasKusama,
  prodParasPolkadot,
  prodParasKusamaCommon,
  prodParasPolkadotCommon,
} from "@polkadot/apps-config/endpoints";

export interface ForeignChainsInfo {
  readonly moonbeamNetworkName: string;
  readonly moonbeamParaId: number;
  readonly foreignChains: ReadonlyArray<ForeignChainInfo>;
}

export interface ForeignChainInfo {
  readonly name: string;
  readonly paraId: number;
  readonly mutedUntil?: number | false;
  readonly endpoints?: readonly string[];
}

export const getEndpoints = (relay: "Polkadot" | "Kusama" | "Unsupported", paraId: number) => {
  switch (relay) {
    case "Polkadot": {
      if (paraId < 2000) {
        const commonGoodPolka = prodParasPolkadotCommon.find((a) => a.paraId === paraId);
        return Object.values(commonGoodPolka!.providers);
      }
      const polkaPara = prodParasPolkadot.find((a) => a.paraId === paraId);
      return Object.values(polkaPara!.providers);
    }
    case "Kusama": {
      if (paraId < 2000) {
        const commonGoodKusama = prodParasKusamaCommon.find((a) => a.paraId === paraId);
        return Object.values(commonGoodKusama!.providers);
      }
      const kusamaPara = prodParasKusama.find((a) => a.paraId === paraId);
      return Object.values(kusamaPara!.providers);
    }
    case "Unsupported":
      throw new Error("Unsupported chain.");
  }
};

export const isMuted = (moonbeamNetworkName: MoonbeamNetworkName, paraId: ParaId) => {
  const info = ForeignChainsEndpoints.find((a) => a.moonbeamNetworkName === moonbeamNetworkName);

  if (info) {
    const chains = info.foreignChains as ReadonlyArray<ForeignChainInfo>;
    const match = chains.find((chain) => chain.paraId === paraId);

    if (!match) {
      console.error(`⚠️  No static data for ParaId ${paraId}, please add to foreign-chains.ts`);
      return false;
    }

    const currentTime = new Date().getTime();
    return match.mutedUntil && match.mutedUntil >= currentTime;
  } else return false;
};

export const ForeignChainsEndpoints = [
  {
    moonbeamNetworkName: "Moonriver",
    moonbeamParaId: 2023,
    foreignChains: [
      {
        name: "Statemine",
        paraId: 1000,
        mutedUntil: false,
      },
      {
        name: "Karura",
        paraId: 2000,
      },
      {
        name: "Bifrost",
        paraId: 2001,
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
        mutedUntil: new Date("2023-11-09").getTime(),
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
      {
        name: "Mangata",
        paraId: 2110,
      },
      {
        name: "Turing",
        paraId: 2114,
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
        mutedUntil: false,
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
        name: "Equilibrium",
        paraId: 2011,
      },
      {
        name: "Parallel",
        paraId: 2012,
      },
      {
        name: "Nodle",
        paraId: 2026,
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
        name: "HydraDX",
        paraId: 2034,
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
        name: "Manta",
        paraId: 2104,
      },
    ],
  },
] satisfies ReadonlyArray<ForeignChainsInfo>;

type ValueOf<T> = T extends readonly (infer U)[] ? U : never;
export type MoonbeamNetworkName = ValueOf<typeof ForeignChainsEndpoints>["moonbeamNetworkName"];

type ElementOf<T> = T extends readonly (infer U)[] ? U : never;
type ForeignChainInfoType = ElementOf<typeof ForeignChainsEndpoints>["foreignChains"][number];
export type ParaId = ForeignChainInfoType["paraId"];
