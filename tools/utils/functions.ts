import pMap from "p-map";

export const promiseConcurrent = <T, R>(
  concurrency: number,
  mapper: (item: T, index?: number) => Promise<R> | R,
  list: T[]
): Promise<R[]> => pMap(list, mapper, { concurrency: concurrency });
