import yargs from "yargs";

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run ts-node github/generate-gh-issue-client-release.ts [args]")
    .version("1.0.0")
    .options({
      from: {
        type: "string",
        describe: "previous client version",
        required: true,
      },
      to: {
        type: "string",
        describe: "next client version",
        required: true,
      },
    })
    .demandOption(["from", "to"])
    .help().argv;

  const previousVersion = argv.from;
  const newVersion = argv.to;

  const commonTemplate = `
  - [ ] Start the github action Publish Binary Draft with ${previousVersion} => ${newVersion}
  (master branch).
  - [ ] Review the generated Draft and clean a bit the messages if needed (keep it draft).
  - [ ] Update moonbeam-networks stagenet (moonsama/moonlama) config.json to include sha-xxxxx
  (matching your ${newVersion} tag) and increase the config version + 1.
  - [ ] Test the new client on stagenet (moonsama/moonlama).
  - [ ] Publish the client release draft.
  - [ ] When everything is ok, publish the new docker image: start github action Publish Docker
  with ${newVersion}.
  - [ ] Publish the new tracing image: on repo moonbeam-runtime-overrides, start github action
  Publish Docker with ${newVersion} and master.
`;

  // Detect if it's a major release or hotfix
  if (newVersion.endsWith(".0")) {
    const template = `
## Requirements
- [ ] To be manually edited (add pending PRs)

## Pre-Release
- [ ] Create a PR that update the launch.ts configuration (to add client conf for this version).
- [ ] Get that PR approved and merged.
- [ ] Re-run all extrinsics/hooks benchmarks.

## Release
- [ ] Tag master with ${newVersion} and push to github
${commonTemplate}

## Post Release
- [ ] Bump client version to the next one on master
    `;
    console.log(template);
  } else {
    const template = `
## Requirements
- [ ] To be manually edited (add pending PRs)

## Pre-Release
- [ ] Create branch \`perm-${newVersion}\` against previous client git tag.
- [ ] In the branch \`perm-${newVersion}\`, bump client version to ${newVersion}.

## Release
- [ ] Tag \`perm-${newVersion}\` with ${newVersion} and push to github.
${commonTemplate}
    `;
    console.log(template);
  }
}

main();
