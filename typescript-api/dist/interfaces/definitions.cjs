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

// src/moonbeam/interfaces/definitions.ts
var definitions_exports = {};
__export(definitions_exports, {
  moon: () => definitions_default
});
module.exports = __toCommonJS(definitions_exports);

// src/moonbeam/interfaces/moon/definitions.ts
var definitions_default = {
  types: {},
  rpc: {
    isBlockFinalized: {
      description: "Returns whether an Ethereum block is finalized",
      params: [{ name: "blockHash", type: "Hash" }],
      type: "bool"
    },
    isTxFinalized: {
      description: "Returns whether an Ethereum transaction is finalized",
      params: [{ name: "txHash", type: "Hash" }],
      type: "bool"
    },
    getLatestSyncedBlock: {
      description: "Returns the latest synced block from Frontier's backend",
      params: [],
      type: "u32"
    }
  }
};
// Annotate the CommonJS export names for ESM import in node:
0 && (module.exports = {
  moon
});
