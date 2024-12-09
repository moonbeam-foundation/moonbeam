// src/moonbeam/tsconfig.json
var extends2 = "../../tsconfig.base.json";
var compilerOptions = {
  rootDir: ".",
  baseUrl: "./",
  outDir: "../../dist",
  declarationDir: "../../dist/types",
  declarationMap: true,
  paths: {
    "@moonbeam/api-augment/*": ["src/moonbeam/*"],
    "@polkadot/api/augment": ["src/moonbeam/interfaces/augment-api.ts"],
    "@polkadot/types/augment": ["src/moonbeam/interfaces/augment-types.ts"],
    "@polkadot/types/lookup": ["src/moonbeam/interfaces/types-lookup.ts"]
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
