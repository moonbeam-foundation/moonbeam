import yargs from "yargs";
import { Octokit } from "octokit";

const octokit = new Octokit();
const labels = {};

// Typescript 4 will support it natively, but not yet :(
type Await<T> = T extends PromiseLike<infer U> ? U : T;
type Commits = Await<ReturnType<Octokit["rest"]["repos"]["compareCommits"]>>["data"]["commits"];

async function getLabels(repo: string, previousTag: string, newTag: string, filters: string[]) {
  let commits: Commits = [];
  let more = true;
  let page = 0;
  while (more) {
    const compare = await octokit.rest.repos.compareCommits({
      owner: repo.split("/")[0],
      repo: repo.split("/")[1],
      base: previousTag,
      head: newTag,
      per_page: 200,
      page,
    });
    commits = commits.concat(compare.data.commits);
    more = compare.data.commits.length == 200;
    page++;
  }

  console.log(
    `found ${commits.length} total commits in https://github.com/${repo}/compare/${previousTag}...${newTag}`
  );

  for (const commit of commits) {
    const prs = await octokit.rest.repos.listPullRequestsAssociatedWithCommit({
      owner: repo.split("/")[0],
      repo: repo.split("/")[1],
      commit_sha: commit.sha,
    });
    for (const pr of prs.data) {
      for (const label of pr.labels) {
        labels[label.name] = labels[label.name] || [];
        labels[label.name].push(`(${repo}#${pr.number}) ${pr.title}`);
      }
    }
  }

  const filterRegs = filters && filters.map((f) => new RegExp(f));
  for (const labelName of Object.keys(labels).sort().reverse()) {
    if (filterRegs && !filterRegs.some((f) => f.test(labelName))) {
      continue;
    }
    console.log(`===== ${labelName}`);
    for (const label of labels[labelName]) {
      console.log(`  ${label}`);
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
        choices: ["paritytech/substrate", "paritytech/polkadot"],
        describe: "which repository to read",
      },
      "only-label": {
        type: "array",
        describe: "filter specific labels (using grep)",
      },
    })
    .demandOption(["from", "to", "repo"])
    .help().argv;

  getLabels(argv.repo, argv.from, argv.to, argv["only-label"] as string[]);
}

main();
