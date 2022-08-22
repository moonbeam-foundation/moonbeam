// Sort dict by key
export function sortObjectByKeys(o) {
  return Object.keys(o)
    .sort()
    .reduce((r, k) => ((r[k] = o[k]), r), {});
}
