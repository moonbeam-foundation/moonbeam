import type { Log, Web3 } from "web3";

type LogsSubscription = Awaited<ReturnType<Web3["eth"]["subscribe"]>>;

export async function web3SubscribeHistoricalLogs(
  web3: Web3,
  listenPeriodMs: number,
  filter?: object
) {
  const eventLogs: Log[] = [];
  const sub = await web3.eth.subscribe("logs", filter);

  await new Promise(async (resolve, reject) => {
    const timeoutId = setTimeout(async () => {
      await sub.unsubscribe();
      resolve("finished");
    }, listenPeriodMs);

    sub.on("data", async (event) => {
      eventLogs.push(event);
    });

    sub.on("error", (error) => {
      clearTimeout(timeoutId);
      console.log("Error when subscribing to New block header: ", error);
      sub.unsubscribe().catch((err) => console.error("Error unsubscribing:", err));
      reject(error);
    });
  });

  return eventLogs;
}

export async function openWeb3LogsSub(web3: Web3, filter?: object): Promise<LogsSubscription> {
  return await web3.eth.subscribe("logs", filter);
}

// Note: Uses any for logSub as web3 v4 has complex union types for subscriptions
export async function onceWeb3Log(logSub: {
  once: (event: string, cb: (log: Log) => void) => void;
}): Promise<Log> {
  return new Promise<Log>((resolve) => {
    logSub.once("data", resolve);
  });
}
