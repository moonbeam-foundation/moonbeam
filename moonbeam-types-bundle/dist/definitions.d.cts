import { OverrideModuleType, OverrideBundleDefinition, OverrideBundleType } from '@polkadot/types/types';

declare const moduleDefinitions: Record<string, OverrideModuleType>;
declare const moonbeamDefinitions: OverrideBundleDefinition;
declare const moonbeamDefinitionsDeprecated: OverrideBundleDefinition;
declare const typesBundlePre900: OverrideBundleType;
declare const typesBundleDeprecated: OverrideBundleType;
declare const types: OverrideBundleType;

export { moduleDefinitions, moonbeamDefinitions, moonbeamDefinitionsDeprecated, types, typesBundleDeprecated, typesBundlePre900 };
