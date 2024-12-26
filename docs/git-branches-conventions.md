# Git branch conventions

There are 3 types of branches:

1. The master branch
2. Development branches (to be merged into master and then deleted)
3. Releases branches

If your contribution is a hotfix (bug fix on a release already published), follow the
[hotfix workflow](#hotfix-workflow) section.
In all other cases, you must create a development branch, and submit a pull request to
master (possibly in "draft" state until you are done).

## Development branches

Development branches must be prefixed with a short name moniker (e.g. `gav-my-feature`).
Development branches must be merged into master and then deleted.
If you have to leave your development for a long time, we invite you to push your branch on your
fork and to remove it from the official repository, so that the latter is not polluted by
too many branches.

## Releases branches

This kind of branch should be named `perm-TAG`, where `TAG` is the git tag of the corresponding
release.

## Hotfix workflow

To fix a bug in a release already published (in github releases page), please follow these steps:

1. Create a release branch dedicated to the hotfix release, based on the tag of the last published
version.
1. Push a commit that bumps the version on the release branch.
1. Submit a PR against `master` to fix the bug upstream.
1. Once the fix is merged on master, cherry-pick the merge commit on the release branch.
1. Make the git tag on the release branch and follow the delivery process.

### Hotfix workflow example runtime hotfix for runtime 1501. 

1. Create the release branch `perm-runtime-1502`, based on tag `runtime-1501`
1. Push a commit on `perm-runtime-1502` that bumps spec versions from 1501 to 1502.
1. Submit a PR against `master` to fix the bug upstream.
1. Once the fix is merged on master, cherry-pick the merge commit on the `perm-runtime-1502` branch.
1. Make the git tag on the `perm-runtime-1502` branch and follow the delivery process.
