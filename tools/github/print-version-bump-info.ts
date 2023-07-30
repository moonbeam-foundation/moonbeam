import { Octokit } from "octokit";
import yargs from "yargs";
import { getCommitAndLabels } from "./github-utils";

async function printInfo(octokit: Octokit, previousVersion: string, nextVersion: string) {
  const owners = {
    substrate: "paritytech",
    polkadot: "paritytech",
    cumulus: "paritytech",
    nimbus: "moonbeam-foundation",
  };
  const prefixes = {
    substrate: "polkadot-",
    polkadot: "release-",
    cumulus: "polkadot-",
    nimbus: "moonbeam-polkadot-",
  };
  console.log(`# Description\n`);
  console.log(`This ticket is automatically generated using\n`);
  console.log("```");
  console.log(`$ npm run print-version-bump-info -- --from ${previousVersion} --to ${nextVersion}`);
  console.log("```");

  const prInfoByLabels = {};
  for (const repo of Object.keys(prefixes)) {
    const previousTag = `${prefixes[repo]}${previousVersion}`;
    const nextTag = `${prefixes[repo]}${nextVersion}`;
    try {
      const previousCommit = await octokit.rest.git.getCommit({
        owner: owners[repo],
        repo,
        commit_sha: (
          await octokit.rest.git.getTree({
            owner: owners[repo],
            repo,
            tree_sha: previousTag,
          })
        ).data.sha,
      });
      const nextCommit = await octokit.rest.git.getCommit({
        owner: owners[repo],
        repo,
        commit_sha: (
          await octokit.rest.git.getTree({
            owner: owners[repo],
            repo,
            tree_sha: nextTag,
          })
        ).data.sha,
      });
      console.log(
        `\n## ${repo} (${previousCommit.data.author.date.slice(
          0,
          10
        )} -> ${nextCommit.data.author.date.slice(0, 10)})\n`
      );
      const { commits, prByLabels } = await getCommitAndLabels(
        octokit,
        owners[repo],
        repo,
        previousTag,
        nextTag
      );
      console.log(`https://github.com/${owners[repo]}/${repo}/compare/${previousTag}...${nextTag}`);
      console.log("```");
      console.log(`    from: ${previousCommit.data.sha}`);
      console.log(`      to: ${nextCommit.data.sha}`);
      console.log(` commits: ${commits.length}`);
      console.log("```");

      for (const label of Object.keys(prByLabels)) {
        prInfoByLabels[label] = (prInfoByLabels[label] || []).concat(
          prByLabels[label].map((pr) => {
            return `  ${`(${owners[repo]}/${repo}#${pr.number}) ${pr.title}`}`;
          })
        );
      }
    } catch (e) {
      console.trace(`Failing to query ${repo} [${previousTag}..${nextTag}]: ${e.toString()}`);
      process.exit(1);
    }
  }

  console.log(`\n# Important commits by label\n`);
  const excludeRegs = [
    /D5-nicetohaveaudit/,
    /D3-trivia/,
    /D2-notlive/,
    /D1-audited/,
    /C[01234]-/,
    /B0-silent/,
    /A[0-9]-/,
  ];
  for (const labelName of Object.keys(prInfoByLabels).sort().reverse()) {
    if (excludeRegs.some((f) => f.test(labelName))) {
      continue;
    }
    console.log(`\n### ${labelName || "N/A"}\n`);
    // Deduplicate PRs on same label
    const deduplicatePrsOfLabel = prInfoByLabels[labelName].filter(function (elem, index, self) {
      return index === self.indexOf(elem);
    });
    for (const prInfo of deduplicatePrsOfLabel) {
      console.log(prInfo);
    }
  }

  console.log(`\n## Review 'substrate-migrations' repo\n`);
  console.log(`https://github.com/apopiak/substrate-migrations#frame-migrations`);
  console.log(`\nThis repository contains a list of FRAME-related migrations which might be`);
  console.log(`relevant to Moonbeam.`);
}

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run print-version-deps [args]")
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
    })
    .demandOption(["from", "to"])
    .help().argv;

  const octokit = new Octokit({
    auth: process.env.GITHUB_TOKEN || undefined,
  });

  printInfo(octokit, argv.from, argv.to);
}

main();
