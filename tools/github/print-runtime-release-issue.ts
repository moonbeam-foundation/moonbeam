import yargs from "yargs";

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run ts-node github/generate-gh-issue-runtime-release.ts [args]")
    .version("1.0.0")
    .options({
      from: {
        type: "string",
        describe: "previous runtime spec version",
        required: true,
      },
      to: {
        type: "string",
        describe: "next runtime spec version",
        required: true,
      },
      client: {
        type: "string",
        describe: "current client version",
        required: true,
      },
    })
    .demandOption(["from", "to", "client"])
    .help().argv;

  const previousVersion = argv.from;
  const newVersion = argv.to;
  const lastClientVersion = argv.client;

  const commonTemplate =
    `
## Release
- [ ] Check all proxy types.
- [ ] Re-run all extrinsics/hooks benchmarks.
- [ ] Tag master with runtime-${newVersion} and push to github
- [ ] Start the github action Publish Runtime Draft
with runtime-${previousVersion} => runtime-${newVersion}
  - \`gh workflow run "Publish Runtime Draft" -r 'master' ` +
    `-f from=runtime-${previousVersion} -f to=runtime-${newVersion}\`
- [ ] Review the generated Draft and clean a bit the messages if needed (keep it draft)
- [ ] Create the tracing runtime on moonbeam-runtime-overrides
(see https://github.com/moonbeam-foundation/moonbeam-runtime-overrides/blob/master/README.md)
- [ ] Upgrade typescript API: Start the github action "Upgrade typescript API"
- [ ] Add new tracing substitute in network configuration
- [ ] Upgrade stagenet
- [ ] Create new tracing image for partners: start the github action Publish Docker
with ${lastClientVersion} and master
- [ ] Upgrade alphanet
- [ ] When everything is ok, publish the draft release
  `;

  // Detect if it's a major release or hotfix
  if (newVersion.endsWith("00")) {
    const template =
      `
## Requirements
- [ ] To be manually edited (add pending PRs)
- [ ] Ready for release

## Pre-Release
- [ ] Cleanup previous migrations (
  https://github.com/moonbeam-foundation/moonbeam/blob/master/runtime/common/src/migrations.rs)
- [ ] Check that proxy types are adapted to extrinsics changes (
  read all PR descriptions with B7-runtimenoteworthy)
- [ ] Re-run all extrinsics/hooks benchmarks.
- [ ] Run \`subxt-diff\` GH workflow to generate RT diffs and merge PR after manual review 👀

${commonTemplate}

## Post Release
- [ ] Publish the docker runtime image (trigger the github action "Publish Docker runtime")
  - \`gh workflow run "Publish Runtime Draft" -r 'master' ` +
      `-f from=runtime-${previousVersion} -f to=runtime-${newVersion}\`
- [ ] Create a PR that increment spec version (like #1051)
    `;
    console.log(template);
  } else {
    const template = `
## Requirements
- [ ] To be manually edited (add pending PRs)
- [ ] Ready for release

## Pre-Release
- [ ] Bump spec version to ${newVersion}
- [ ] Run \`subxt-diff\` GH workflow to generate RT diffs and merge PR after manual review 👀

${commonTemplate}

## Post Release
- [ ] Publish the docker runtime image (trigger the github action "Publish Docker runtime")
    `;
    console.log(template);
  }
}

main();
