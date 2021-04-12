import tcpPortUsed from "tcp-port-used";

export async function findAvailablePorts() {
  const availablePorts = await Promise.all(
    [null, null, null].map(async (_, index) => {
      let selectedPort = 0;
      let port = 1024 + index * 20000 + (process.pid % 20000);
      let endingPort = 65535;
      while (!selectedPort && port < endingPort) {
        const inUse = await tcpPortUsed.check(port, "127.0.0.1");
        if (!inUse) {
          selectedPort = port;
        }
        port++;
      }
      if (!selectedPort) {
        throw new Error(`No available port`);
      }
      return selectedPort;
    })
  );

  return {
    p2pPort: availablePorts[0],
    rpcPort: availablePorts[1],
    wsPort: availablePorts[2],
  };
}
