// src/moonriver/tsconfig.json
var extends2 = "../../tsconfig.base.json";
var compilerOptions = {
  rootDir: ".",
  baseUrl: "./",
  outDir: "../../dist/moonriver",
  declarationDir: "../../dist/moonriver/types",
  declarationMap: true,
  paths: {
    "@moonbeam/api-augment/moonriver/*": ["src/moonriver/*"],
    "@polkadot/api/augment": ["src/moonriver/interfaces/augment-api.ts"],
    "@polkadot/types/augment": ["src/moonriver/interfaces/augment-types.ts"],
    "@polkadot/types/lookup": ["src/moonriver/interfaces/types-lookup.ts"]
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
