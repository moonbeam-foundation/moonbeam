import { execSync } from "child_process";
import { writeFileSync } from "fs";

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

function getRuntimeInfo(runtimeName: string) {
  const specVersion = execSync(
    `cat ../runtime/${runtimeName}/src/lib.rs | grep 'spec_version: [0-9]*' | tail -1`
  ).toString();
  return {
    name: runtimeName,
    version: /:\s?([0-9A-z\-]*)/.exec(specVersion)[1],
    srtool: require(`../${runtimeName}_srtool_output.json`),
  };
}

const main = () => {
  const lastTags = execSync(
    'git tag | grep "v[0-9]*.[0-9]*.[0-9]*$" | sort -t "." -k1,1n -k2,2n -k3,3n | tail -2'
  )
    .toString()
    .split("\n");

  const previousTag = lastTags[0];
  const newTag = lastTags[1];

  const runtimes = ["moonbase", "moonshadow", "moonriver", "moonbeam"].map((runtimeName) =>
    getRuntimeInfo(runtimeName)
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
* sha256: ${runtime.srtool.sha256}
* size: ${runtime.srtool.size}
* proposal: ${runtime.srtool.prop}
`
  )
  .join(`\n\n`)}

## Build information

WASM runtime built using \`${runtimes[0].srtool.rustc}\`

## Changes

${commits.map((commit) => `* ${commit}`).join("\n")}

## Dependency changes

Moonbeam: https://github.com/PureStake/moonbeam/compare/${previousTag}...${newTag}
${moduleLinks.map((modules) => `${capitalize(modules.name)}: ${modules.link}`).join("\n")}
`;
  console.log(template);
};

main();
