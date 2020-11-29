## Moonbeam Release Process

### Branches

- Releases are taken from the branch `master` using tags `vX.X.X`

### Notes

- The `master` branch must refer to frontier `paritytech/frontier/master` or
  `purestake/frontier/vX.X-hotfixes`

### Pre-Release Checklist

- [ ] Purge requirement has been communicated and scheduled with ops
- [ ] Purge announcement has been prepared and scheduled
- [ ] Documentation/Website/Tutorials have been updated
- [ ] Runtime version has been updated
- [ ] Tests are passing on StageNet
- [ ] Ensure Type changes are reflected into PolkadotJs

### Release Workflow

Below are the steps of the release workflow. Steps prefixed with NOACTION are
automated and require no human action.

1. To initiate the release process, tag the current master.

   - `git checkout master; git pull; git tag -a v0.3.0 -m 'Moonbase v0.3.0'; git push origin v0.3.0`

2. NOACTION: The docker image is built with the given tag
3. NOACTION: Produce release draft including binaries & specs
4. Complete the draft with information from PRs
5. Publish the release

### Post-Release Checklist

- [ ] Release note contains all meaningful notes
- [ ] Release note contains binaries and specs
