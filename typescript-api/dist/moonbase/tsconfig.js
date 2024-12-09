// src/moonbase/tsconfig.json
var extends2 = "../../tsconfig.base.json";
var compilerOptions = {
  rootDir: ".",
  baseUrl: "./",
  outDir: "../../dist/moonbase",
  declarationDir: "../../dist/moonbase/types",
  declarationMap: true,
  paths: {
    "@moonbeam/api-augment/moonbase/*": ["src/moonbase/*"],
    "@polkadot/api/augment": ["src/moonbase/interfaces/augment-api.ts"],
    "@polkadot/types/augment": ["src/moonbase/interfaces/augment-types.ts"],
    "@polkadot/types/lookup": ["src/moonbase/interfaces/types-lookup.ts"]
  }
};
var tsconfig_default = {
  extends: extends2,
  compilerOptions
};
export {
  compilerOptions,
  tsconfig_default as default,
  extends2 as extends
};
