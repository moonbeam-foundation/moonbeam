import { execSync } from "child_process";
import { readFileSync } from "fs";
import yargs from "yargs";
import path from "path";

function capitalize(s) {
  return s[0].toUpperCase() + s.slice(1);
}

function getCompareLink(packageName: string, previousTag: string, newTag: string) {
  const previousPackage = execSync(
    `git show ${previousTag}:../Cargo.lock | grep ${packageName}? | head -1 | grep -o '".*"'`
  ).toString();
  const previosCommit = /#([0-9a-f]*)/g.exec(previousPackage)[1].slice(0, 8);
  const previousRepo = /(https:\/\/.*)\?/g.exec(previousPackage)[1];

  const newPackage = execSync(
    `git show ${newTag}:../Cargo.lock | grep ${packageName}? | head -1 | grep -o '".*"'`
  ).toString();
  const newCommit = /#([0-9a-f]*)/g.exec(newPackage)[1].slice(0, 8);
  const newRepo = /(https:\/\/.*)\?/g.exec(newPackage)[1];
  const newRepoOrganization = /github.com\/([^\/]*)/g.exec(newRepo)[1];

  const diffLink =
    previousRepo !== newRepo
      ? `${previousRepo}/compare/${previosCommit}...${newRepoOrganization}:${newCommit}`
      : `${previousRepo}/compare/${previosCommit}...${newCommit}`;

  return diffLink;
}

function getRuntimeInfo(srtoolReportFolder: string, runtimeName: string) {
  const specVersion = execSync(
    `cat ../runtime/${runtimeName}/src/lib.rs | grep 'spec_version: [0-9]*' | tail -1`
  ).toString();
  return {
    name: runtimeName,
    version: /:\s?([0-9A-z\-]*)/.exec(specVersion)[1],
    srtool: JSON.parse(
      readFileSync(path.join(srtoolReportFolder, `./${runtimeName}-srtool-digest.json`)).toString()
    ),
  };
}

const main = () => {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run ts-node github/generate-release-body.ts [args]")
    .version("1.0.0")
    .options({
      "srtool-report-folder": {
        type: "string",
        describe: "folder which contains <runtime>-srtool-digest.json",
        required: true,
      },
    })
    .demandOption(["srtool-report-folder"])
    .help().argv;

  const lastTags = execSync(
    'git tag | grep "^v[0-9]*.[0-9]*.[0-9]*$" | sort -t "." -k1,1n -k2,2n -k3,3n | tail -2'
  )
    .toString()
    .split("\n");

  const previousTag = lastTags[0];
  const newTag = lastTags[1];

  if (!previousTag || !newTag) {
    console.log(
      `Couldn't retrieve tags from`,
      execSync(
        'git tag | grep "^v[0-9]*.[0-9]*.[0-9]*$" | sort -t "." -k1,1n -k2,2n -k3,3n | tail -2'
      )
        .toString()
        .replace(/\n/g, ", ")
    );
    process.exit(1);
  }

  const runtimes = ["moonbase", "moonshadow", "moonriver", "moonbeam"].map((runtimeName) =>
    getRuntimeInfo(argv["srtool-report-folder"], runtimeName)
  );
  const moduleLinks = ["substrate", "polkadot", "cumulus", "frontier"].map((repoName) => ({
    name: repoName,
    link: getCompareLink(repoName, previousTag, newTag),
  }));
  const commits = execSync(`git log --oneline --pretty=format:"%s" ${previousTag}...${newTag}`)
    .toString()
    .split(`\n`)
    .filter((l) => !!l);

  const template = `
## Runtimes

${runtimes
  .map(
    (runtime) => `### ${capitalize(runtime.name)}

* spec_version: ${runtime.version}
* sha256: ${runtime.srtool.runtimes.compact.sha256}
* size: ${runtime.srtool.runtimes.compact.size}
* proposal: ${runtime.srtool.runtimes.compact.prop}
`
  )
  .join(`\n\n`)}

## Build information

WASM runtime built using \`${runtimes[0].srtool.info.rustc}\`

## Changes

${commits.map((commit) => `* ${commit}`).join("\n")}

## Dependency changes

Moonbeam: https://github.com/PureStake/moonbeam/compare/${previousTag}...${newTag}
${moduleLinks.map((modules) => `${capitalize(modules.name)}: ${modules.link}`).join("\n")}
`;
  console.log(template);
};

main();
