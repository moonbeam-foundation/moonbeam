# Git branch conventions

There are 4 types of branches:

1. The master branch
2. Releases branches
3. Development branches (to be merged into master and then deleted)

If your contribution is a hotfix (bug fix on a release already published), follow the
[hotfix branches](#hotfix-branches) section.
In all other cases, you must create a development branch, and submit a pull request to
master (possibly in "draft" state until you are done).

## Hotfix branches

To fix a bug in a release already published (in github releases page),
the way to proceed will depend on whether this release has already be subject to hotfix or not:

### First hotfix

If you need to make a hotfix on a release that never received one,
you must first create a release branch dedicated to that release
(from the commit pointed by the tag of the concerned release.), this branch must be named
`perm-TAG`, where `TAG` must be replaced by the git tag of the major release.

Then you need to create a branch named `{name}-{short-description}`, where `{name}` is to
be replaced by your nickname, and then create a PR against the release branch.
This procedure allows to play the CI to check the integrity of a hotfix.

Example:

You have to make a first hotfix on the release `v2.3.0` and your nickname is `kevin`. We suppose that
it is for example a bug in the `key` subcommand.

1. Create a branch name `perm-v2.3` from an existing release tag.
2. Create a branch name `kevin-fix-key-command`
3. Create a PR from `kevin-fix-key-command` into `perm-v2.3`.
4. Request an emergency one or more reviews of this PR.
5. Merge this PR.
6. Create a git tag `v2.3.1` on the `perm-v2.3` branch.
7. If relevant, create a companion PR to backport this hotfix into master. This backport should
then be integrated into master via a development branch, as if it were a "classic" contribution.

### Non-first hotfix

If the release branch dedicated to this release does not exist, or is not in the right format,
apply the procedure in the [First hotfix](#first-hotfix) section.

If there is already a release branch for this release named in the format `perm-TAG`,
directly create a branch in the format `{name}-{short-description}` and create a PR from
this branch into `perm-TAG`.

## Development branches

Development branches must be prefixed with a short name moniker (e.g. `gav-my-feature`).
Development branches must be merged into master and then deleted.
If you have to leave your development for a long time, we invite you to push your branch on your
fork and to remove it from the official repository, so that the latter is not polluted by
too many branches.
