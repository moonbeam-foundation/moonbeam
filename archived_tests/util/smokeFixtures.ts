import { SubstrateApi, EthersApi, ApiType } from "./wsApis";
const debug = require("debug")("smoke:mocha-setup");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

export async function mochaGlobalSetup() {
  if (!wssUrl) {
    throw Error(`Missing wssUrl parameter (use WSS_URL=... npm run smoke-test)`);
  }
  await SubstrateApi.api(ApiType.ParaChain, wssUrl);
  await SubstrateApi.api(ApiType.RelayChain, relayWssUrl);
  EthersApi.api(wssUrl);
  debug(`📡  ApiConnections created.`);
}

export function mochaGlobalTeardown() {
  SubstrateApi.close(ApiType.ParaChain);
  SubstrateApi.close(ApiType.RelayChain);
  EthersApi.close();
}
