import { test } from "../util/setup";

test("Moonbeam RPC (Version) - eth_chainId should match", async (t) => {
  t.is(await t.context.web3.eth.getChainId(), 1281);
});

test("Moonbeam RPC (Version) - net_version should match", async (t) => {
  t.is(await t.context.web3.eth.net.getId(), 1281);
});
