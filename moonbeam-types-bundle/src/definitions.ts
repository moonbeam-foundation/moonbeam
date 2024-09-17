import type {
    OverrideBundleDefinition,
    OverrideBundleType,
    OverrideModuleType,
    OverrideVersionedType
} from "@polkadot/types/types";
import { rpcDefinitions } from "./rpc";
import * as Types from "./types";
  
  // override types for specific pallets
  export const moduleDefinitions: Record<string, OverrideModuleType> = {
    assetManager: {
      Balance: "TAssetBalance",
    },
    xTokens: {
      Balance: "TAssetBalance",
    },
  };
  
  export const moonbeamDefinitions = {
    alias: moduleDefinitions,
    rpc: rpcDefinitions,
    instances: {
      council: ["councilCollective"],
      technicalCommittee: ["techCommitteeCollective", "openTechCommitteeCollective"],
    },
    types: [
      {
        minmax: [0, 4],
        types: Types.TYPES_0_4,
      },
      {
        minmax: [5, 5],
        types: Types.TYPES_5_5,
      },
      {
        minmax: [6, 19],
        types: Types.TYPES_6_19,
      },
      {
        minmax: [19, 35],
        types: Types.TYPES_19_35,
      },
      {
        minmax: [36, 36],
        types: Types.TYPES_36_36,
      },
      {
        minmax: [37, 42],
        types: Types.TYPES_37_42,
      },
      {
        minmax: [43, 154],
        types: Types.TYPES_43_154,
      },
      {
        minmax: [155, 199],
        types: Types.TYPES_155_199,
      },
      {
        minmax: [200, 399],
        types: Types.TYPES_200_399,
      },
      {
        minmax: [400, 599],
        types: Types.TYPES_400_599,
      },
      {
        minmax: [600, 799],
        types: Types.TYPES_600_799,
      },
      {
        minmax: [800, 899],
        types: Types.TYPES_800_899,
      },
      {
        minmax: [900, undefined],
        types: Types.TYPES_POST_900,
      },
    ],
  } as OverrideBundleDefinition;
  
  export const moonbeamDefinitionsDeprecated = {
    ...moonbeamDefinitions,
    types: [
      ...(moonbeamDefinitions.types as OverrideVersionedType[]),
      {
        minmax: [900, undefined],
        types: Types.TYPES_900_undefined_deprecated,
      },
    ],
  } as OverrideBundleDefinition;
  
  export const typesBundlePre900 = {
    spec: {
      moonbeam: moonbeamDefinitions,
      moonbeamDefinitions,
      moonbase: moonbeamDefinitions,
      moonriver: moonbeamDefinitions,
    },
  } as OverrideBundleType;
  
  export const typesBundleDeprecated = {
    spec: {
      moonbeam: moonbeamDefinitionsDeprecated,
      moonbeamDefinitions: moonbeamDefinitionsDeprecated,
      moonbase: moonbeamDefinitionsDeprecated,
      moonriver: moonbeamDefinitionsDeprecated,
    },
  } as OverrideBundleType;
  
  // default types to use
  export const types = typesBundlePre900;
  