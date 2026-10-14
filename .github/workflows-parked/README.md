# Parked workflows

The Moonbeam parachain is retired. The workflows that built, tested and released the node and the runtimes ran on paid runners (16-core GitHub runners, Blacksmith and `moonbeam-release-medium`) and on weekly and daily schedules, so they kept costing money with nothing left to ship.

They are kept here unchanged, and GitHub does not run anything in this folder. Dependabot's config is parked here too, so it no longer opens update PRs.

To bring one back, move it into `.github/workflows/` (and `dependabot.yml` back into `.github/`). Actions must also be allowed for this repository in the organization's settings.
