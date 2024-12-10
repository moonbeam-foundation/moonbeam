import type { LogsSubscription } from "node_modules/web3/lib/types/eth.exports.js";
import type { Log, Web3 } from "web3";

export async function web3SubscribeHistoricalLogs(
  web3: Web3,
  listenPeriodMs: number,
  filter?: object
) {
  const eventLogs: any[] = [];
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

export async function onceWeb3Log(logSub: LogsSubscription): Promise<Log> {
  return new Promise<Log>((resolve) => {
    logSub.once("data", resolve);
  });
}
