import tcpPortUsed from "tcp-port-used";
import lockfile from "lockfile";
import fs from "fs";

const LOCKFILE = "/tmp/moonbeam_tests_port_lockfile";
const PORT_EXCLUSION_FILE = "/tmp/moonbeam_tests_used_ports.json";

async function lockLockfile() {

  // https://github.com/npm/lockfile#options
  const lockfileOpts = {
    wait: 30000,
    stale: 600000, // a whopping 10 minutes
  };

  return new Promise((resolve, reject) => {
    lockfile.lock(LOCKFILE, {wait:30000}, (err) => {
      if (err) {
        console.log("Caught error in lock(): ", err);
        reject(err);
      } else {
        resolve();
      }
    });
  });
}

async function unlockLockfile() {
  return new Promise((resolve, reject) => {
    lockfile.unlock(LOCKFILE, (err) => {
      if (err) {
        console.log("Caught error in unlock(): ", err);
        reject(err);
      } else {
        resolve();
      }
    });
  });
}

async function queryUsedPorts() {
  return new Promise<number[]>((resolve, reject) => {
    fs.readFile(PORT_EXCLUSION_FILE, 'utf8', function(err, data) {
      if (err) {
        if (err.code == 'ENOENT') {
          // means file didn't exist -- return empty []
          resolve([]);
        } else {
          console.log("Caught error in readFile(): ", err);
          reject(err);
        }
      } else {
        let ports = JSON.parse(data);
        resolve(ports);
      }
    });
  });
}

async function saveUsedPorts(usedPorts: number[]) {
  return new Promise((resolve, reject) => {
    fs.writeFile(PORT_EXCLUSION_FILE, JSON.stringify(usedPorts), function(err) {
      if (err) {
        console.log("Caught error in readFile(): ", err);
        reject(err);
      } else {
        resolve();
      }
    });
  });
}

export async function findAvailablePorts() {
  const startingPort = 20000;
  const endingPort = 29999;

  // obtain lockfile -- this is used as a sort of multi-process mutex
  // console.log("-------------------------- OBTAINING LOCKFILE...");
  await lockLockfile();
  // console.log("-------------------------- LOCKFILE OBTAINED <<<<<<<<<<<");

  let availablePorts = [];

  try {
    // now open our used-ports file. we will avoid using any ports listed there
    // once we pick ports, we'll write them to this file
    // console.log("------------ READING PORTS FILE...");
    let usedPorts = await queryUsedPorts();
    // TODO: assert that usedPorts is an array

    // console.log("used ports so far: ", usedPorts);

    let port = startingPort;
    while (availablePorts.length < 3 && port < endingPort) {
      // TODO: optimize
      if (usedPorts.indexOf(port) >= 0) {
        // console.log(`skipping already-used port ${port}`);
        port++;
        continue;
      }

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

    await saveUsedPorts([...usedPorts, ...availablePorts]);

  } catch (err) {
    throw err;
  } finally {
    // console.log("-------------------------- RELEASING LOCKFILE...");
    await unlockLockfile();
    // console.log("-------------------------- LOCKFILE RELEASED >>>>>>>>>>>");
  }

  return {
    p2pPort: availablePorts[0],
    rpcPort: availablePorts[1],
    wsPort: availablePorts[2],
  };
}
