# Automation

This section of the documentation is dedicated to the automation processes for the CI.

## Architecture

Automation is using Github Action, where all the actions are described in [.github/workflows](.github/workflows)

### Blacksmith runners

CI jobs run on Blacksmith ephemeral cloud runners. Runner labels such as
`blacksmith-4vcpu-ubuntu-2404` and `blacksmith-16vcpu-ubuntu-2404` select the
machine size used by each action.

## Cancellation

It is possible to cancel actions directly in github action UI or using (replace "build.yml" by the desired
action and "my-branch" by the desired pr branch):  
`gh run list --workflow="build.yml" --json status,headBranch,databaseId | jq '.[] | select(.headBranch == "my-branch" and (.status == "in_progress" or .status == "queued")) | .databaseId' | xargs -n1 --no-run-if-empty gh run cancel`

## Actions

### Cancel

Cancel allows to cancel previous execution of the same action for the same PR in order to release allocated resources. This is the case of
[build](#build) action

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

### Publish runtime draft

[.github/workflows/publish-runtime.yml](.github/workflows/publish-runtime.yml)  
`gh workflow run "Publish Runtime Draft" -r master -f from=runtime-2302 -f to=runtime-2400`

Builds the runtime of each network using srtool and then draft
the release notes using the git commits between the given tags.
The draft is "not published" (TODO: change the name of the action)
