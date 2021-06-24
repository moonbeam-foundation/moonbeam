import { Octokit } from "octokit";

// Typescript 4 will support it natively, but not yet :(
type Await<T> = T extends PromiseLike<infer U> ? U : T;
type Commits = Await<ReturnType<Octokit["rest"]["repos"]["compareCommits"]>>["data"]["commits"];

export async function getCommitAndLabels(
  octokit: Octokit,
  owner: string,
  repo: string,
  previousTag: string,
  newTag: string
) {
  let commits: Commits = [];
  let more = true;
  let page = 0;
  while (more) {
    const compare = await octokit.rest.repos.compareCommits({
      owner,
      repo,
      base: previousTag,
      head: newTag,
      per_page: 200,
      page,
    });
    commits = commits.concat(compare.data.commits);
    more = compare.data.commits.length == 200;
    page++;
  }

  const prByLabels = {};
  for (const commit of commits) {
    const prs = await octokit.rest.repos.listPullRequestsAssociatedWithCommit({
      owner,
      repo,
      commit_sha: commit.sha,
    });
    for (const pr of prs.data) {
      if (pr.labels && pr.labels.length > 0) {
        for (const label of pr.labels) {
          prByLabels[label.name] = prByLabels[label.name] || [];
          prByLabels[label.name].push(pr);
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
