"use strict";
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __export = (target, all) => {
  for (var name in all)
    __defProp(target, name, { get: all[name], enumerable: true });
};
var __copyProps = (to, from, except, desc) => {
  if (from && typeof from === "object" || typeof from === "function") {
    for (let key of __getOwnPropNames(from))
      if (!__hasOwnProp.call(to, key) && key !== except)
        __defProp(to, key, { get: () => from[key], enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable });
  }
  return to;
};
var __toCommonJS = (mod) => __copyProps(__defProp({}, "__esModule", { value: true }), mod);

// src/moonriver/interfaces/definitions.ts
var definitions_exports = {};
__export(definitions_exports, {
  moon: () => definitions_default
});
module.exports = __toCommonJS(definitions_exports);

// src/moonriver/interfaces/moon/definitions.ts
var import_moonbeam_types_bundle = require("moonbeam-types-bundle");
var _a;
var definitions_default = {
  types: {},
  rpc: {
    ...(_a = import_moonbeam_types_bundle.moonbeamDefinitions.rpc) == null ? void 0 : _a.moon
  }
};
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  moon
});
