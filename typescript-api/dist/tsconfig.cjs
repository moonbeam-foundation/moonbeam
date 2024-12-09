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
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  compilerOptions,
  extends: null
});
