import yargs from "yargs";
import { BN } from "@polkadot/util";

const args = yargs.options({
  "yes": { type: "number", demandOption: true, alias: "y" },
  "no": { type: "number", demandOption: true, alias: "n" },
  "turnout": { type: "number", demandOption: true, alias: "t" },
  "electorate": { type: "number", demandOption: true, alias: "e" },
  "approveType": {
    choices: ["SuperMajorityApprove", "SuperMajorityAgainst", "SimpleMajority"],
    demandOption: true,
    alias: "at",
  },
}).argv;

function compareRationals(n1: BN, d1: BN, n2: BN, d2: BN): boolean {
  // Uses a continued fractional representation for a non-overflowing compare.
  // Detailed at https://janmr.com/blog/2014/05/comparing-rational-numbers-without-overflow/.
  while (true) {
    const q1 = n1.div(d1);
    const q2 = n2.div(d2);
    if (q1.lt(q2)) {
      return true;
    }
    if (q2.lt(q1)) {
      return false;
    }
    const r1 = n1.mod(d1);
    const r2 = n2.mod(d2);
    if (r2.isZero()) {
      return false;
    }
    if (r1.isZero()) {
      return true;
    }
    n1 = d2;
    n2 = d1;
    d1 = r2;
    d2 = r1;
  }
}

async function main() {
  const yes = args["yes"];
  const no = args["no"];
  const sqrtVoters = Math.sqrt(args["turnout"]);
  const sqrtElectorate = Math.sqrt(args["electorate"]);

  if (sqrtVoters == 0) {
    console.log("Result is", false);
    return;
  }

  let result = false;
  if (args["approveType"] == "SuperMajorityApprove") {
    result = compareRationals(new BN(no), new BN(sqrtVoters), new BN(yes), new BN(sqrtElectorate))
  }
  else if(args["approveType"] == "SuperMajorityAgainst") {
    result =  compareRationals(new BN(no), new BN(sqrtElectorate), new BN(yes), new BN(sqrtVoters))
  }
  else{
    result = yes > no
  }

  console.log("Result is", result);
}

main()
  .catch(console.error)
  .finally(() => process.exit());
