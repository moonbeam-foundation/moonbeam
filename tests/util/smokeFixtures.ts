import { SubstrateApi, EthersApi, ApiType } from "./wsApis";
const debug = require("debug")("smoke:mocha-setup");

const wssUrl = process.env.WSS_URL || null;
const relayWssUrl = process.env.RELAY_WSS_URL || null;

export async function mochaGlobalSetup() {
  await SubstrateApi.api(ApiType.ParaChain, wssUrl);
  if (relayWssUrl) {
    await SubstrateApi.api(ApiType.RelayChain, relayWssUrl);
  }
  EthersApi.api(wssUrl);
  debug(`ApiConnections created.`);
}

export function mochaGlobalTeardown() {
  SubstrateApi.close(ApiType.ParaChain);
  SubstrateApi.close(ApiType.RelayChain);
  EthersApi.close();
}
