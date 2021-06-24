import { Octokit } from "octokit";
import yargs from "yargs";
import { getCommitAndLabels } from "./github-utils";

async function listPrByLabels(
  octokit: Octokit,
  owner: string,
  repo: string,
  previousTag: string,
  newTag: string,
  filters: string[],
  excludeList: string[]
) {
  const { commits, prByLabels } = await getCommitAndLabels(
    octokit,
    owner,
    repo,
    previousTag,
    newTag
  );
  const filterRegs = filters && filters.map((f) => new RegExp(f));
  const excludeRegs = excludeList && excludeList.map((f) => new RegExp(f));

  console.log(
    `found ${commits.length} total commits in ` +
      `https://github.com/${owner}/${repo}/compare/${previousTag}...${newTag}`
  );

  for (const labelName of Object.keys(prByLabels).sort().reverse()) {
    if (filterRegs && !filterRegs.some((f) => f.test(labelName))) {
      continue;
    }
    if (excludeRegs && excludeRegs.some((f) => f.test(labelName))) {
      continue;
    }
    console.log(`===== ${labelName}`);
    for (const pr of prByLabels[labelName]) {
      console.log(`  ${`(${owner}/${repo}#${pr.number}) ${pr.title}`}`);
    }
  }
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run list-pr-labels [args]")
    .version("1.0.0")
    .options({
      from: {
        type: "string",
        describe: "commit-sha/tag of range start",
      },
      to: {
        type: "string",
        describe: "commit-sha/tag of range end",
      },
      repo: {
        type: "string",
        choices: ["paritytech/substrate", "paritytech/polkadot", "paritytech/cumulus"],
        describe: "which repository to read",
      },
      "only-label": {
        type: "array",
        describe: "keep only specific labels (using grep)",
      },
      "exclude-label": {
        type: "array",
        alias: "e",
        describe: "exclude specific labels (using grep)",
      },
    })
    .demandOption(["from", "to", "repo"])
    .help().argv;

  const octokit = new Octokit({
    auth: process.env.GITHUB_TOKEN || undefined,
  });

  listPrByLabels(
    octokit,
    argv.repo.split("/")[0],
    argv.repo.split("/")[1],
    argv.from,
    argv.to,
    argv["only-label"] as string[],
    argv["exclude-label"] as string[]
  );
}

main();
