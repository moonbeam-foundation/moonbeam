import yargs from "yargs";
import { Keyring } from "@polkadot/api";
import { hexToU8a, u8aToHex } from "@polkadot/util";
import { cryptoWaitReady } from "@polkadot/util-crypto";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0")
  .version("1.0.0")
  .options({
    "private-key": { type: "string", demandOption: true },
    "reward-address": { type: "string", demandOption: true },
    strength: { type: "number", default: 256 },
  })
  .check((argv) => {
    // Basic check on the ethereum address
    if (!/^(0x){1}[0-9a-fA-F]{40}$/i.test(argv["reward-address"])) {
      throw new Error(`Invalid Ethereum reward address: ${argv["reward-address"]}`);
    }
    return true;
  }).argv;

const privateKey = argv["private-key"];
const rewardAddress = argv["reward-address"];

const main = async () => {
  await cryptoWaitReady();
  const keyring = new Keyring({ type: "sr25519" });

  const relayAccount = keyring.addFromUri(privateKey);

  const message = hexToU8a(rewardAddress);
  const signature = relayAccount.sign(message);

  console.log(
    `Moonriver Crowdloan Reward
Address: ${rewardAddress} 
Signature: ${u8aToHex(signature)}`
  );
};

main();
