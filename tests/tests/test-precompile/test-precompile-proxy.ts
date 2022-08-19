import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS } from "../../util/accounts";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";
import {
  CONTRACT_PROXY_TYPE_ANY,
  CONTRACT_PROXY_TYPE_GOVERNANCE,
  CONTRACT_PROXY_TYPE_STAKING,
  PRECOMPILE_PROXY_ADDRESS,
} from "../../util/constants";
import { expectEVMResult } from "../../util/eth-transactions";
import { web3EthCall } from "../../util/providers";

const PROXY_CONTRACT_JSON = getCompiled("Proxy");
const PROXY_INTERFACE = new ethers.utils.Interface(PROXY_CONTRACT_JSON.contract.abi);

describeDevMoonbeam("Precompile - Proxy - add proxy fails if pre-existing proxy", (context) => {
  before("add proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });

  it("should fail re-adding proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });
});

describeDevMoonbeam("Precompile - Proxy - add proxy succeeds with valid account", (context) => {
  it("should add proxy", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");

    const proxyAddedEvents = result.events.reduce((acc, e) => {
      if (context.polkadotApi.events.proxy.ProxyAdded.is(e.event)) {
        acc.push({
          account: e.event.data[0].toString(),
          proxyType: e.event.data[2].toHuman(),
        });
      }
      return acc;
    }, []);

    expect(proxyAddedEvents).to.deep.equal([
      {
        account: ALITH_ADDRESS,
        proxyType: "Staking",
      },
    ]);
  });
});

describeDevMoonbeam("Precompile - Proxy - remove proxy fails if no existing proxy", (context) => {
  it("should fail removing proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("removeProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Revert");
  });
});

describeDevMoonbeam("Precompile - Proxy - remove proxy succeeds if existing proxy", (context) => {
  before("add proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });

  it("should fail re-adding proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("removeProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");

    const proxyRemovedEvents = result.events.reduce((acc, e) => {
      if (context.polkadotApi.events.proxy.ProxyRemoved.is(e.event)) {
        acc.push({
          account: e.event.data[0].toString(),
          proxyType: e.event.data[2].toHuman(),
        });
      }
      return acc;
    }, []);

    expect(proxyRemovedEvents).to.deep.equal([
      {
        account: ALITH_ADDRESS,
        proxyType: "Staking",
      },
    ]);
  });
});

describeDevMoonbeam(
  "Precompile - Proxy - remove proxies succeeds even if no existing proxy",
  (context) => {
    it("should fail removing proxy account", async () => {
      const { result } = await context.createBlock(
        createTransaction(context, {
          ...ALITH_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_PROXY_ADDRESS,
          data: PROXY_INTERFACE.encodeFunctionData("removeProxies"),
        })
      );
      expectEVMResult(result.events, "Succeed");
    });
  }
);

describeDevMoonbeam("Precompile - Proxy - remove proxies succeeds if existing proxy", (context) => {
  before("add 2 proxy accounts", async () => {
    const { result: resultBaltathar } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(resultBaltathar.events, "Succeed");

    const { result: resultCharleth } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          CHARLETH_ADDRESS,
          CONTRACT_PROXY_TYPE_GOVERNANCE,
          0,
        ]),
      })
    );
    expectEVMResult(resultCharleth.events, "Succeed");
  });

  it("should remove all proxy accounts", async () => {
    const proxiesBefore = (
      await context.polkadotApi.query.proxy.proxies(ALITH_ADDRESS)
    )[0].toJSON();
    expect(proxiesBefore).to.have.lengthOf(2);

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("removeProxies"),
      })
    );
    expectEVMResult(result.events, "Succeed");

    const proxiesAfter = (await context.polkadotApi.query.proxy.proxies(ALITH_ADDRESS))[0].toJSON();
    expect(proxiesAfter).to.be.empty;
  });
});

describeDevMoonbeam("Precompile - Proxy - is proxy - fails if incorrect delay", (context) => {
  before("add proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });

  it("should return false", async () => {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PROXY_ADDRESS,
      data: PROXY_INTERFACE.encodeFunctionData("isProxy", [
        ALITH_ADDRESS,
        BALTATHAR_ADDRESS,
        CONTRACT_PROXY_TYPE_STAKING,
        1,
      ]),
    });
    expect(Number(result)).to.equal(0);
  });
});

describeDevMoonbeam("Precompile - Proxy - is proxy - fails if incorrect proxyType", (context) => {
  before("add proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });

  it("should return false", async () => {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PROXY_ADDRESS,
      data: PROXY_INTERFACE.encodeFunctionData("isProxy", [
        ALITH_ADDRESS,
        BALTATHAR_ADDRESS,
        CONTRACT_PROXY_TYPE_ANY,
        0,
      ]),
    });
    expect(Number(result)).to.equal(0);
  });
});

describeDevMoonbeam("Precompile - Proxy - is proxy - succeeds if exists", (context) => {
  before("add proxy account", async () => {
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_STAKING,
          0,
        ]),
      })
    );
    expectEVMResult(result.events, "Succeed");
  });

  it("should return true", async () => {
    const { result } = await web3EthCall(context.web3, {
      to: PRECOMPILE_PROXY_ADDRESS,
      data: PROXY_INTERFACE.encodeFunctionData("isProxy", [
        ALITH_ADDRESS,
        BALTATHAR_ADDRESS,
        CONTRACT_PROXY_TYPE_STAKING,
        0,
      ]),
    });
    expect(Number(result)).to.equal(1);
  });
});
