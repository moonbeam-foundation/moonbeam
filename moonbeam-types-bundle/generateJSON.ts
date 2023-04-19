// @ts-expect-error
import type { RegistryTypes } from "@polkadot/types/types";
import fs from "fs";
import { moonbeamDefinitions } from ".";

async function generateJSON() {
  const version = process.argv[2] || "latest";
  let types: RegistryTypes;
  if (!moonbeamDefinitions.types) {
    throw new Error("missing types definitions");
  } else if (version === "latest") {
    types = moonbeamDefinitions.types[moonbeamDefinitions.types.length - 1].types;
  } else if (Number(version)) {
    let i = 0;
    while (
      i < moonbeamDefinitions.types.length &&
      moonbeamDefinitions.types[i].minmax[1] &&
      Number(moonbeamDefinitions.types[i].minmax[1]) < Number(version)
    ) {
      i += 1;
    }
    types = moonbeamDefinitions.types[i].types;
  } else {
    throw new Error("parameter must be number or `latest`");
  }
  console.log(JSON.stringify(types));
  fs.appendFile("moonbeam-types-" + version + ".json", JSON.stringify(types), function (err) {
    if (err) throw err;
    console.log("Saved for version : " + version);
  });
}
generateJSON();
