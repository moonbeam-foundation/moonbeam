import "@polkadot/api-augment";
import { startParachainNodes } from "./util/para-node";

const SPEC_FILE = process.env.SPEC_FILE;
const PARA_ID = process.env.PARA_ID && parseInt(process.env.PARA_ID);

if (!SPEC_FILE) {
  console.error(`Missing SPEC_FILE (ex: ~/exports/moonbeam-state.mod.json)`);
  process.exit(1);
}

if (!PARA_ID) {
  console.error(`Missing PARA_ID (ex: 2004)`);
  process.exit(1);
}

startParachainNodes({
  parachain: {
    spec: SPEC_FILE,
    binary: "local",
  },
  paraId: PARA_ID,
  relaychain: {
    binary: "local",
  },
});
