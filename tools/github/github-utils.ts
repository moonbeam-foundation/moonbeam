import { Octokit } from "octokit";
import { execSync } from "child_process";

// Typescript 4 will support it natively, but not yet :(
type Await<T> = T extends PromiseLike<infer U> ? U : T;
type Commits = Await<ReturnType<Octokit["rest"]["repos"]["compareCommits"]>>["data"]["commits"];

export function getCompareLink(packageName: string, previousTag: string, newTag: string) {
  const previousPackage = execSync(
    `git show ${previousTag}:../Cargo.lock | grep ${packageName}? | head -1 | grep -o '".*"'`
  ).toString();
  const previousCommit = /#([0-9a-f]*)/g.exec(previousPackage)[1].slice(0, 8);
  const previousRepo = /(https:\/\/.*)\?/g.exec(previousPackage)[1];

  const newPackage = execSync(
    `git show ${newTag}:../Cargo.lock | grep ${packageName}? | head -1 | grep -o '".*"'`
  ).toString();
  const newCommit = /#([0-9a-f]*)/g.exec(newPackage)[1].slice(0, 8);
  const newRepo = /(https:\/\/.*)\?/g.exec(newPackage)[1];
  const newRepoOrganization = /github.com\/([^\/]*)/g.exec(newRepo)[1];

  const diffLink =
    previousRepo !== newRepo
      ? `${previousRepo}/compare/${previousCommit}...${newRepoOrganization}:${newCommit}`
      : `${previousRepo}/compare/${previousCommit}...${newCommit}`;

  return diffLink;
}

export async function getCommitAndLabels(
  octokit: Octokit,
  owner: string,
  repo: string,
  previousTag: string,
  newTag: string
): Promise<{ prByLabels: any; commits: any[] }> {
  let commits: Commits = [];
  let more = true;
  let page = 0;
  while (more) {
    const compare = await octokit.rest.repos.compareCommitsWithBasehead({
      owner,
      repo,
      basehead: previousTag + "..." + newTag,
      per_page: 200,
      page,
    });
    commits = commits.concat(compare.data.commits);
    more = compare.data.commits.length == 200;
    page++;
  }

  const prByLabels = {};
  for (const commit of commits) {
    const foundPrsNumbers = commit.commit.message.split('\n')[0].match(/\(#([0-9]+)\)/);
    if (foundPrsNumbers && foundPrsNumbers.length > 1) {
      const pr = await octokit.rest.pulls.get({
        owner,
        repo,
        pull_number: parseInt(foundPrsNumbers[1]),
      });

      if (pr.data.labels && pr.data.labels.length > 0) {
        for (const label of pr.data.labels) {
          prByLabels[label.name] = prByLabels[label.name] || [];
          prByLabels[label.name].push(pr.data);
        }
      } else {
        prByLabels[""] = prByLabels[""] || [];
        prByLabels[""].push(pr);
      }
    }
  }
  return {
    prByLabels,
    commits,
  };
}
