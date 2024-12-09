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
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  compilerOptions,
  extends: null
});
