# Automation

This section of the documentation is dedicated to the automation processes for the CI.

## Architecture

Automation is using Github Action, where all the actions are described in [.github/workflows](.github/workflows)

### bare-metal

label bare-metal refers to our CI servers managed by opslayer. Those are dedicated machines, optimized to reduce the compilation and testing time of the actions.

## Cancellation

It is possible to cancel actions directly in github action UI or using (replace "coverage.yml" by the desired
action and "my-branch" by the desired pr branch):  
`gh run list --workflow="coverage.yml" --json status,headBranch,databaseId | jq '.[] | select(.headBranch == "my-branch" and (.status == "in_progress" or .status == "queued")) | .databaseId' | xargs -n1 --no-run-if-empty gh run cancel`

## Actions

### Cancel

Cancel allows to cancel previous execution of the same action for the same PR in order to release allocated resources. This is the case of
[build](#build) and [coverage](#coverage) actions

### Build

[.github/workflows/build.yml](.github/workflows/build.yml)  
`gh workflow run build -r my-branch-or-tag`

Performs multiple actions to ensure the code provided is high quality:

- Checks formatting/copyrights/links/locks/...
- Builds the runtime and binary (with and without features)
- Prepare external binaries (polkadot) matching the new code
- Tests:
  - Rust tests (mostly unit), including tracing features
  - Typescript (mostly dev integrations), including full relay chain & tracing
  - Fork & Dev upgrade tests
- Build a docker image based on the sha and push to docker.io

### Coverage

[.github/workflows/coverage.yml](.github/workflows/coverage.yml)  
`gh workflow run coverage -r my-branch-or-tag`

Similar to [build](#build), coverage ensure the quality of our code
and test by re-building our binary with `grcov` and re-executing our tests.

The output is stored as an artefact and published in the comments of the PR.

### Publish runtime draft

[.github/workflows/publish-runtime.yml](.github/workflows/publish-runtime.yml)  
`gh workflow run "Publish Runtime Draft" -r master -f from=runtime-2302 -f to=runtime-2400`

Builds the runtime of each network using srtool and then draft
the release notes using the git commits between the given tags.
The draft is "not published" (TODO: change the name of the action)
