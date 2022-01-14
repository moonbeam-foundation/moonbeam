import yargs from "yargs";

async function main() {
  const argv = yargs(process.argv.slice(2))
    .usage("Usage: npm run ts-node github/generate-gh-issue-runtime-release.ts [args]")
    .version("1.0.0")
    .options({
      from: {
        type: "string",
        describe: "previous runtime version",
        required: true,
      },
      to: {
        type: "string",
        describe: "next runtime version",
        required: true,
      },
      client: {
        type: "string",
        describe: "current client version",
        required: true,
      },
    })
    .demandOption(["from", "to"])
    .help().argv;

  const previousVersion = argv.from;
  const newVersion = argv.to;
  const lastClientVersion = argv.client;

  const template = `
  - [ ] Cleanup previous migrations (only for major release,
  https://github.com/PureStake/moonbeam/blob/master/runtime/common/src/migrations.rs).
  - [ ] Create a PR that increment spec version (like #1051).
  - [ ] Get that PR approved and merged.
  - [ ] Tag master with runtime-${newVersion} and push to github.
  - [ ] Start the github action \`Publish Runtime Draft\`.
  with params \`runtime-${previousVersion}\` and \`runtime-${newVersion}\`.
  - [ ] Create the tracing runtime PR on git repo \`purestake/moonbeam-runtime-overrides\`.
  - [ ] Get the tracing runtime PR approved and merged.
  - [ ] Review the generated Draft and clean a bit the messages if needed (keep it draft)
  (see https://github.com/PureStake/moonbeam-runtime-overrides/blob/master/README.md).
  - [ ] Try the runtime upgrade in a local network (with launch script).
  - [ ] Add new tracing substitute in stagenet/moonsama/moonlama configuration.
  - [ ] Upgrade stagenet.
  - [ ] Add new tracing substitute in alphanet configuration.
  - [ ] Upgrade alphanet.
  - [ ] Create new tracing image for partners: start the github action \`Publish Docker\`
  with params \`${lastClientVersion}\` and \`master\` on git repo \`purestake/moonbeam-runtime-overrides\`.
  - [ ] When everything is ok, publish the draft release.
  `;

  console.log(template);
}

main();
