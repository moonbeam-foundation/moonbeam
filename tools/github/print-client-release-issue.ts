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

  const template = `
  - [ ] Create a PR with v${newVersion} (see README.md last section or last v${previousVersion} PR to see the required changed)
  - [ ] Get that PR approved and merged
  - [ ] Tag master with v${newVersion} and push to github
  - [ ] Start the github action Publish Binary Draft with v${previousVersion} => v${newVersion}
  - [ ] Review the generated Draft and clean a bit the messages if needed (keep it draft until I approve)
  - [ ] Update moonbeam-networks stagenet/moonsama config.json to include sha-xxxxx (matching your v${newVersion} tag) and increase the config version + 1
  - [ ] Test the new client on stagenet and moonsama
  - [ ] When everything is ok, publish the new docker image: start github action Publish Docker with v${newVersion}
  - [ ] Publish the new tracing image: on repo moonbeam-runtime-overrides, start github action Publish Docker with v${newVersion} and master
  `;

  console.log(template);
}

main();
