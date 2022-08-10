#!/usr/bin/env ts-node

import { ApiPromise, WsProvider } from "@polkadot/api";
import { u8aToHex, hexToU8a } from "@polkadot/util";
import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import yargs from "yargs";
import mergeWith from "lodash.mergewith";
import isObject from "lodash.isobject";

const args = yargs
  .showHelpOnFail(true)
  .command(
    "key",
    "Compute storage key",
    {
      pallet: {
        type: "string",
        demandOption: true,
        alias: "pallet",
        describe: "The pallet's capitalized name",
      },
      storage: {
        type: "string",
        demandOption: true,
        alias: "storage",
        describe: "The storage's capitalized name",
      },
      key: {
        type: "string",
        demandOption: true,
        alias: "key",
        describe: "The key for the storage",
      },
    },
    cmdKey
  )
  .command(
    "encode",
    "Encode JSON value to SCALE",
    {
      ws: { type: "string", demandOption: true, describe: "The websocket URL" },
      type: { type: "string", demandOption: true, describe: "The type name" },
      value: { type: "string", demandOption: true, describe: "The JSON or Raw value for the type" },
      explain: { type: "boolean", describe: "Explain the output" },
    },
    cmdEncode
  )
  .command(
    "decode",
    "Decode SCALE hex value to object",
    {
      ws: { type: "string", demandOption: true, describe: "The websocket URL" },
      type: { type: "string", demandOption: true, describe: "The type name" },
      value: { type: "string", demandOption: true, describe: "The JSON or Raw value for the type" },
      explain: { type: "boolean", describe: "Explain the output" },
    },
    cmdDecode
  )
  .help()
  .demandCommand().argv;

async function cmdKey(argv: any) {
  let palletEncoder = new TextEncoder().encode(argv["pallet"]);
  let palletHash = xxhashAsU8a(palletEncoder, 128);
  let storageEncoder = new TextEncoder().encode(argv["storage"]);
  let storageHash = xxhashAsU8a(storageEncoder, 128);
  let key = new Uint8Array([...hexToU8a(argv["key"])]);
  let keyHash = blake2AsU8a(key, 128);

  console.log(u8aToHex(new Uint8Array([...palletHash, ...storageHash, ...keyHash, ...key])));
}

async function cmdEncode(argv: any) {
  const api = await ApiPromise.create({
    initWasm: false,
    provider: new WsProvider(argv["ws"]),
  });

  try {
    const obj = api.createType(argv["type"] as any, JSON.parse(argv["value"]));
    const encodingExplained = explainEncoding(obj.inspect());
    if (argv["explain"]) {
      console.log(recursiveMerge(obj.toHuman(), encodingExplained));
    } else {
      console.log(u8aToHex(obj.toU8a()));
    }

    process.exit(0);
  } catch (e) {
    await api.disconnect();
    throw e;
  }
}

async function cmdDecode(argv: any) {
  const api = await ApiPromise.create({
    initWasm: false,
    provider: new WsProvider(argv["ws"]),
  });

  try {
    const obj = api.createType(argv["type"] as any, hexToU8a(argv["value"]));
    const encodingExplained = explainEncoding(obj.inspect());
    if (argv["explain"]) {
      console.log(recursiveMerge(obj.toHuman(), encodingExplained));
    } else {
      console.log(obj.toHuman());
    }
    process.exit(0);
  } catch (e) {
    await api.disconnect();
    throw e;
  }
}

function explainEncoding(o: any, scaleObj: any = {}) {
  if (o.name) {
    if (o.outer) {
      scaleObj[o.name] = u8aToHex(o.outer[0], null, false);
    }
    if (o.inner) {
      for (const inner of o.inner) {
        scaleObj[o.name] = explainEncoding(inner, scaleObj[o.name]);
      }
    }

    return scaleObj;
  }

  if (o.outer) {
    return u8aToHex(o.outer[0], null, false);
  }

  if (o.inner && o.inner.length > 0) {
    const isObject = !!o.inner[0].name;
    if (isObject) {
      for (const inner of o.inner) {
        scaleObj = explainEncoding(inner, scaleObj);
      }
    } else {
      scaleObj = [];
      for (const inner of o.inner) {
        scaleObj.push(explainEncoding(inner));
      }
    }
  }

  return scaleObj;
}

function recursiveMerge(a: any, b: any) {
  if (isObject(a) && isObject(b)) {
    return mergeWith(a, b, recursiveMerge);
  }

  return {
    value: a,
    scale: b,
  };
}
