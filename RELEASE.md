# Moonbeam Release Process

## Moonbeam client release

### Branches

- Releases are taken from the branch `master` using tags `vX.X.X`

### Notes

- The `master` branch must refer to frontier `paritytech/frontier/master` or
  `moonbeam-foundation/frontier/moonbeam-polkadot-vX.Y.Z`

### Release Workflow

To release `vX.Y.Z`:

1. Create a PR with an increment client version
1. Get that PR approved and merged
1. Tag master with `vX.Y.Z` and push to GitHub
1. Start the GitHub action "Publish Binary Draft" (on master branch)
1. Review the generated Draft and clean a bit the messages if needed (keep it draft)
1. Test the new client on internal tests networks (stagenet/moonsama/moonlama)
1. Publish the client release draft
1. When everything is ok, publish the new docker image: start GitHub action Publish Docker with
   `vX.Y.Z`
1. Publish the new tracing image: on repo moonbeam-runtime-overrides, start GitHub action
   Publish Docker with `vX.Y.Z` and master
1. Documentation/Website/Tutorials have been updated

## Moonbeam runtime release

### Branches

- Releases are taken from the branch `master` using tags `runtime-XXYY`
- If the master branch contains changes that should not be included in the next runtime, then
  create a `perm-runtime-XXYY` branch and create the `runtime-XXYY` tag on that branch.

### Release Workflow

To release `runtime-XXYY`:

1. Create a PR that increment spec version (like #1051)
1. Get that PR approved and merged
1. Tag master with `runtime-XXYY` and push to GitHub
1. Start the GitHub action "Publish Runtime Draft"
1. Review the generated Draft and clean a bit the messages if needed (keep it draft)
1. Create the tracing runtime: start the GitHub action "Create tracing runtime" on `moonbeam-foundation/moonbeam-runtime-overrides`
1. Upgrade runtime on our internal test network stagenet
1. Ensure Type changes are reflected into PolkadotJs
1. Test changes on stagenet
1. Create new tracing image for partners: start the GitHub action "Publish Docker"
   on `moonbeam-foundation/moonbeam-runtime-overrides`
1. When everything is ok, publish the draft release
1. Upgrade runtime on alphanet
