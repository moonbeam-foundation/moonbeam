---
name: qa-cherry-picks
description: QA cherry-pick tracking documents for polkadot-sdk fork upgrades. Use when auditing, verifying, or updating cherry-pick tables in docs/cherry-picks/, when upgrading to a new polkadot-sdk stable branch, or when checking if a cherry-pick was applied.
---

# QA Cherry-Picks

Moonbeam maintains forks of `polkadot-sdk`, `frontier`, `evm`, `ethereum`, and `moonkit`. Each fork has a `moonbeam-polkadot-stable<YYMM>` branch with cherry-picks on top of the upstream stable release. The cherry-pick tracking documents in `docs/cherry-picks/` record every cherry-pick and its status.

## Documents

- `docs/cherry-picks/polkadot-sdk-stable2512.md` — current
- `docs/cherry-picks/polkadot-sdk-stable2506.md` — previous

## Table Schema

Each repo section has a markdown table with these columns:

| Column      | Description                                                                                                              |
|-------------|--------------------------------------------------------------------------------------------------------------------------|
| Applied     | `Yes`/`No` — was it applied on the previous stable branch?                                                               |
| Title       | Human-readable description of the change                                                                                 |
| Commit      | Link to the actual commit on the current `moonbeam-polkadot-stable<YYMM>` branch (empty if Dropped)                      |
| Cherry pick | `Included` (on current branch), `Dropped` (removed), `squash` (folded into another)                                      |
| Status      | Upstream status: `Permanent`, `Temporary`, `PR Upstream Merged`, `Upstream PR not merged`, `Needs PR upstream`, `Closed` |
| Upstream PR | Link to the upstream PR (if any)                                                                                         |
| Note        | Context, Jira tickets, related PRs                                                                                       |

## Verifying Cherry-Picks Against a Branch

For each repo (e.g., `polkadot-sdk`), find the moonbeam-only commits:

```bash
cd ../polkadot-sdk
git fetch upstream stable2512
MERGE_BASE=$(git merge-base origin/moonbeam-polkadot-stable2512 upstream/stable2512)
git log --format="%H %s" $MERGE_BASE..origin/moonbeam-polkadot-stable2512
```

Upstream remotes by repo:

| Repo         | Origin (fork)                        | Upstream                                             |
|--------------|--------------------------------------|------------------------------------------------------|
| polkadot-sdk | moonbeam-foundation/polkadot-sdk     | paritytech/polkadot-sdk (branch: `stable<YYMM>`)     |
| frontier     | moonbeam-foundation/frontier         | polkadot-evm/frontier (tag: `frontier-stable<YYMM>`) |
| evm          | moonbeam-foundation/evm              | rust-ethereum/evm (branch: `master`)                 |
| ethereum     | moonbeam-foundation/ethereum         | rust-ethereum/ethereum (branch: `master`)            |
| moonkit      | moonbeam-foundation/moonkit (origin) | Moonsong-Labs/moonkit (branch: `main`)               |

Then for each "Included" row, confirm the commit exists on the branch. For each "Dropped" row, confirm it does not.

## Creating a New Cherry-Pick Document

When upgrading to a new stable branch (e.g., `stable2603`):

1. **Copy the previous doc** as a starting point.
2. **For each "Included" item**, check if it's still needed:
   - Search for the upstream PR — if merged into the new stable, mark as `Dropped`.
   - Otherwise, find the new commit hash on the new branch and update the Commit column.
3. **For each "Dropped" item from the previous doc**, keep it for historical reference.
4. **Check for gaps** — diff the previous doc's "Included" items against the new doc to catch anything forgotten:
   - Items absorbed into upstream need no action.
   - Items with `Temporary` or `Needs PR upstream` status need careful review.
5. **Verify every row** by running the merge-base method above on each repo.

## Common Pitfalls

- **Commit hashes change between branches** — the same logical cherry-pick has different hashes on `stable2506` vs `stable2512`. Always look up the new hash by commit message or content.
- **Merge commits** — some cherry-picks land as merge PRs (e.g., `Merge pull request #8`). Check the diff, not just the subject line.
- **Cross-repo entries** — the original Notion export occasionally placed frontier cherry-picks in the polkadot-sdk table. Flag these.
- **"Applied" vs "Cherry pick"** — `Applied` refers to the *previous* branch; `Cherry pick` refers to the *current* branch. An item can be `Applied=Yes, Cherry pick=Dropped` (was on old branch, removed from new one because upstream merged it).
- **EVM fork divergence** — upstream `rust-ethereum/evm` did a v1.0 rewrite. Moonbeam is on the 0.43.x fork, so "PR Upstream Merged" doesn't mean the cherry-pick can be dropped.
