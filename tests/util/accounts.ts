import Keyring from "@polkadot/keyring";

import {
  DEFAULT_GENESIS_BALANCE,
  DEFAULT_GENESIS_MAPPING,
  DEFAULT_GENESIS_STAKING,
} from "./constants";

const keyringEth = new Keyring({ type: "ethereum" });
const keyringEd25519 = new Keyring({ type: "ed25519" });
const keyringSr25519 = new Keyring({ type: "sr25519" });

// Prefunded accounts.
export const ALITH_ADDRESS = "0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac";
export const ALITH_PRIVATE_KEY =
  "0x5fb92d6e98884f76de468fa3f6278f8807c48bebc13595d45af5bdc4da702133";
export const ALITH_SESSION_ADDRESS =
  "0xd43593c715fdd31c61141abd04a99fd6822c8558854ccde39a5684e7a56da27d";
export const ALITH_CONTRACT_ADDRESSES = [
  "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3",
  "0x970951a12F975E6762482ACA81E57D5A2A4e73F4",
  "0x3ed62137c5DB927cb137c26455969116BF0c23Cb",
  "0x962c0940d72E7Db6c9a5F81f1cA87D8DB2B82A23",
  "0x5CC307268a1393AB9A764A20DACE848AB8275c46",
  "0xeAB4eEBa1FF8504c124D031F6844AD98d07C318f",
];

export const BALTATHAR_ADDRESS = "0x3Cd0A705a2DC65e5b1E1205896BaA2be8A07c6e0";
export const BALTATHAR_PRIVATE_KEY =
  "0x8075991ce870b93a8870eca0c0f91913d12f47948ca0fd25b49c6fa7cdbeee8b";
export const BALTATHAR_SESSION_ADDRESS =
  "0x8eaf04151687736326c9fea17e25fc5287613693c912909cb226aa4794f26a48";

export const CHARLETH_ADDRESS = "0x798d4Ba9baf0064Ec19eB4F0a1a45785ae9D6DFc";
export const CHARLETH_PRIVATE_KEY =
  "0x0b6e18cafb6ed99687ec547bd28139cafdd2bffe70e6b688025de6b445aa5c5b";
export const CHARLETH_SESSION_ADDRESS =
  "0x90b5ab205c6974c9ea841be688864633dc9ca8a357843eeacf2314649965fe22";

export const DOROTHY_ADDRESS = "0x773539d4Ac0e786233D90A233654ccEE26a613D9";
export const DOROTHY_PRIVATE_KEY =
  "0x39539ab1876910bbf3a223d84a29e28f1cb4e2e456503e7e91ed39b2e7223d68";

export const ETHAN_ADDRESS = "0xFf64d3F6efE2317EE2807d223a0Bdc4c0c49dfDB";
export const ETHAN_PRIVATE_KEY =
  "0x7dce9bc8babb68fec1409be38c8e1a52650206a7ed90ff956ae8a6d15eeaaef4";

export const FAITH_ADDRESS = "0xC0F0f4ab324C46e55D02D0033343B4Be8A55532d";
export const FAITH_PRIVATE_KEY =
  "0xb9d2ea9a615f3165812e8d44de0d24da9bbd164b65c4f0573e1ce2c8dbd9c8df";

export const GOLIATH_ADDRESS = "0x7BF369283338E12C90514468aa3868A551AB2929";
export const GOLIATH_PRIVATE_KEY =
  "0x96b8a38e12e1a31dee1eab2fffdf9d9990045f5b37e44d8cc27766ef294acf18";

// Deprecated
export const GERALD_ADDRESS = "0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b";
export const GERALD_PRIVATE_KEY =
  "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
export const GERALD_CONTRACT_ADDRESSES = [
  "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
  "0x5c4242beB94dE30b922f57241f1D02f36e906915",
  "0x42e2EE7Ba8975c473157634Ac2AF4098190fc741",
  "0xF8cef78E923919054037a1D03662bBD884fF4edf",
  "0xe573BCA813c741229ffB2488F7856C6cAa841041",
  "0xBb0CC0fb3e0c06725c67167501f850B4900D6DB5",
];

// This is Alice0x6Be02d1d3665660d22FF9624b7BE0551ee1Ac91b
export const ALITH_GENESIS_FREE_BALANCE = DEFAULT_GENESIS_BALANCE - DEFAULT_GENESIS_MAPPING;
export const ALITH_GENESIS_LOCK_BALANCE = DEFAULT_GENESIS_STAKING;
export const ALITH_GENESIS_RESERVE_BALANCE = DEFAULT_GENESIS_MAPPING;
export const ALITH_GENESIS_TRANSFERABLE_BALANCE =
  ALITH_GENESIS_FREE_BALANCE - ALITH_GENESIS_LOCK_BALANCE;

export const alith = keyringEth.addFromUri(ALITH_PRIVATE_KEY);
export const baltathar = keyringEth.addFromUri(BALTATHAR_PRIVATE_KEY);
export const charleth = keyringEth.addFromUri(CHARLETH_PRIVATE_KEY);
export const dorothy = keyringEth.addFromUri(DOROTHY_PRIVATE_KEY);
export const ethan = keyringEth.addFromUri(ETHAN_PRIVATE_KEY);
export const faith = keyringEth.addFromUri(FAITH_PRIVATE_KEY);
export const goliath = keyringEth.addFromUri(GOLIATH_PRIVATE_KEY);

// deprecated
export const gerald = keyringEth.addFromUri(GERALD_PRIVATE_KEY);

let accountSeed = 10000;
export function generateKeyringPair(type: "ethereum" | "sr25519" | "ed25519" = "ethereum") {
  const privateKey = `0xDEADBEEF${(accountSeed++).toString(16).padStart(56, "0")}`;
  if (type == "sr25519") {
    return keyringSr25519.addFromUri(privateKey);
  } else if (type == "ed25519") {
    return keyringEd25519.addFromUri(privateKey);
  }
  return keyringEth.addFromUri(privateKey);
}
