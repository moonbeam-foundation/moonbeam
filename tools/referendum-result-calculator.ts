import yargs from "yargs";
import { BN, bnSqrt } from "@polkadot/util";
import { calcPassing } from "@polkadot/api-derive/democracy/util";

const args = yargs.options({
  yes: { type: "string", demandOption: true, alias: "y" },
  no: { type: "string", demandOption: true, alias: "n" },
  turnout: { type: "string", demandOption: true, alias: "t" },
  electorate: { type: "string", demandOption: true, alias: "e" },
  voteThreshold: {
    choices: ["isSuperMajorityApprove", "isSuperMajorityAgainst", "isSimpleMajority"],
    demandOption: true,
    alias: "vt",
  },
}).argv;

async function main() {
  const yes = new BN(args["yes"]);
  const no = new BN(args["no"]);
  const voters = new BN(args["turnout"]);
  const sqrtElectorate = bnSqrt(new BN(args["electorate"]));

  let result = calcPassing(args["voteThreshold"] as any, sqrtElectorate, {
    votedAye: yes,
    votedNay: no,
    votedTotal: voters,
  });

  console.log("Result is", result);
}

main()
  .catch(console.error)
  .finally(() => process.exit());
