import tcpPortUsed from "tcp-port-used";

export async function findAvailablePorts() {
  const startingPort = 20000;
  const endingPort = 29999;

  let port = startingPort;
  let availablePorts = [];
  while (availablePorts.length < 3 && port < endingPort) {
    const inUse = await tcpPortUsed.check(port, "127.0.0.1");
    try {
      if (!inUse) {
        availablePorts.push(port);
      }

      port++;
    } catch (err) {
      console.log(`Error checking port ${port}: `, err);
      port++;
    }
  }

  if (availablePorts.length < 3) {
    throw `Could not find 3 ports available within range [${startingPort}, ${endingPort}]`;
  }

  return {
    p2pPort: availablePorts[0],
    rpcPort: availablePorts[1],
    wsPort: availablePorts[2],
  };
}
