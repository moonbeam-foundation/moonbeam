import { expect } from "chai";
import { ethers } from "ethers";
import { describeDevMoonbeam, DevTestContext } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import {
  GENESIS_ACCOUNT,
  ALITH,
  BALTATHAR,
  ALITH_PRIV_KEY,
  CHARLETH,
  BALTATHAR_PRIV_KEY,
} from "../util/constants";
import { createTransaction } from "../util/transactions";
import { getCompiled } from "../util/contracts";
import { ApiPromise, Keyring } from "@polkadot/api";
import { KeyringPair } from "@polkadot/keyring/types";
import { createBlockWithExtrinsic } from "../util/substrate-rpc";
import { BN } from "@polkadot/util";

// const ADDRESS_ERC20 = "0x0000000000000000000000000000000000000802";
// const SELECTORS = {
//   balanceOf: "70a08231",
//   totalSupply: "18160ddd",
//   approve: "095ea7b3",
//   allowance: "dd62ed3e",
//   transfer: "a9059cbb",
//   transferFrom: "23b872dd",
//   logApprove: "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
//   logTransfer: "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
// };
function getAssetAddress(assetId: string) {
  if (assetId.length !== 32) {
    throw new Error("asset id is not 32 long");
  }
  return `0xFFFFFFFF${assetId}`;
}
const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

async function getBalance(context, blockHeight, address) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free;
}

async function sendApprove(context, from, fromPrivate, spender, amount) {
  const fromData = from.slice(2).padStart(64, "0").toLowerCase();
  const spenderData = spender.slice(2).padStart(64, "0").toLowerCase();

  const tx = await createTransaction(context.web3, {
    from: from,
    privateKey: fromPrivate,
    value: "0x0",
    gas: "0x200000",
    gasPrice: GAS_PRICE,
    to: ADDRESS_ERC20,
    data: `0x${SELECTORS.approve}${spenderData}${amount}`,
  });

  const block = await context.createBlock({
    transactions: [tx],
  });

  const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
  expect(receipt.status).to.equal(true);
  expect(receipt.logs.length).to.eq(1);
  expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
  expect(receipt.logs[0].data).to.eq(`0x${amount}`);
  expect(receipt.logs[0].topics.length).to.eq(3);
  expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
  expect(receipt.logs[0].topics[1]).to.eq(`0x${fromData}`);
  expect(receipt.logs[0].topics[2]).to.eq(`0x${spenderData}`);
}

async function checkAllowance(context, owner, spender, amount) {
  const ownerData = owner.slice(2).padStart(64, "0");
  const spenderData = spender.slice(2).padStart(64, "0");

  const request = await customWeb3Request(context.web3, "eth_call", [
    {
      from: GENESIS_ACCOUNT,
      value: "0x0",
      gas: "0x10000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_ERC20,
      data: `0x${SELECTORS.allowance}${ownerData}${spenderData}`,
    },
  ]);

  expect(request.result).equals(`0x${amount.padStart(64, "0")}`);
}

interface AssetMetadata {
  name: string;
  symbol: string;
  decimals: BN;
  isFrozen: boolean;
}
async function registerAssetToParachain(context: DevTestContext, sudoKeyring: KeyringPair) {
  const sourceLocationRelay = { XCM: { parents: 1, interior: "Here" } };
  const relayAssetMetadata: AssetMetadata = {
    name: "DOT",
    symbol: "DOT",
    decimals: new BN(12),
    isFrozen: false,
  };
  const { events: eventsRegister } = await createBlockWithExtrinsic(
    context,
    sudoKeyring,
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.assetManager.registerAsset(
        sourceLocationRelay,
        relayAssetMetadata,
        new BN(1)
      )
    )
  );
  let assetId: string;
  // Look for assetId in events
  eventsRegister.forEach((e) => {
    let ev = e.toHuman();
    if (ev.section === "assetManager") {
      assetId = ev.data[0];
    }
  });
  if (!assetId) {
    await new Promise((res) => setTimeout(res, 20000));
  }
  assetId = assetId.replace(/,/g, "");

  // setAssetUnitsPerSecond
  const { events } = await createBlockWithExtrinsic(
    context,
    sudoKeyring,
    context.polkadotApi.tx.sudo.sudo(
      context.polkadotApi.tx.assetManager.setAssetUnitsPerSecond(assetId, 0)
    )
  );
  return { events, assetId };
}

describeDevMoonbeam("Precompiles - Assets", (context) => {
  let iFace;
  let contractData;
  before("Deploy contract", async () => {
    contractData = await getCompiled("XtokensInstance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
  });
  it("allows to call getBalance", async function () {
    const keyring = new Keyring({ type: "ethereum" });
    const alith = await keyring.addFromUri(ALITH_PRIV_KEY, null, "ethereum");
    const data = iFace.encodeFunctionData("balanceOf", [ALITH]);
    // const address = ALITH.slice(2).padStart(64, "0");

    const { assetId } = await registerAssetToParachain(context, alith);
    console.log("asset id", assetId, assetId.length);

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: getAssetAddress(assetId),
        data, //: `0x${SELECTORS.balanceOf}${address}`,
      },
    ]);

    let amount = await getBalance(context, 0, ALITH);
    amount = "0x" + amount.toHex().slice(2).padStart(64, "0");
    expect(tx_call.result).equals(amount);
  });

  it("allows to call totalSupply", async function () {
    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_ERC20,
        data: `0x${SELECTORS.totalSupply}`,
      },
    ]);

    const amount = await context.polkadotApi.query.balances.totalIssuance();
    const amount_hex = "0x" + amount.toHex().slice(2).padStart(64, "0");

    console.log({ tx_call });

    expect(tx_call.result).equals(amount_hex);
  });
});

describeDevMoonbeam("Precompiles - ERC20 Native", (context) => {
  it("allows to approve transfers, and allowance matches", async function () {
    const amount = `1000000000000`.padStart(64, "0");

    await sendApprove(context, ALITH, ALITH_PRIV_KEY, BALTATHAR, amount);

    await checkAllowance(context, ALITH, BALTATHAR, amount);
  });
});

describeDevMoonbeam("Precompiles - ERC20 Native", (context) => {
  it("allows to call transfer", async function () {
    const amount = `400000000000`.padStart(64, "0");

    const to = CHARLETH.slice(2).padStart(64, "0");
    const tx = await createTransaction(context.web3, {
      from: ALITH,
      privateKey: ALITH_PRIV_KEY,
      value: "0x0",
      gas: "0x200000",
      gasPrice: GAS_PRICE,
      to: ADDRESS_ERC20,
      data: `0x${SELECTORS.transfer}${to}${amount}`,
    });

    const block = await context.createBlock({
      transactions: [tx],
    });

    const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
    expect(receipt.status).to.equal(true);

    const fees = receipt.gasUsed * 1_000_000_000;

    expect(BigInt(await getBalance(context, 1, ALITH))).to.equal(
      BigInt(await getBalance(context, 0, ALITH)) - BigInt(`0x${amount}`) - BigInt(fees)
    );
    expect(BigInt(await getBalance(context, 1, CHARLETH))).to.equal(
      BigInt(await getBalance(context, 0, CHARLETH)) + BigInt(`0x${amount}`)
    );
  });
});

describeDevMoonbeam("Precompiles - ERC20 Native", (context) => {
  it("allows to approve transfer and use transferFrom", async function () {
    const allowedAmount = `1000000000000`.padStart(64, "0");
    const transferAmount = `400000000000`.padStart(64, "0");

    await sendApprove(context, ALITH, ALITH_PRIV_KEY, BALTATHAR, allowedAmount);

    // transferFrom
    {
      const from = ALITH.slice(2).padStart(64, "0").toLowerCase();
      const to = CHARLETH.slice(2).padStart(64, "0").toLowerCase();

      const tx = await createTransaction(context.web3, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_ERC20,
        data: `0x${SELECTORS.transferFrom}${from}${to}${transferAmount}`,
      });

      const block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);

      console.log({ receipt });

      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address).to.eq(ADDRESS_ERC20);
      expect(receipt.logs[0].data).to.eq(`0x${transferAmount}`);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
      expect(receipt.logs[0].topics[1]).to.eq(`0x${from}`);
      expect(receipt.logs[0].topics[2]).to.eq(`0x${to}`);

      expect(receipt.status).to.equal(true);
    }

    expect(BigInt(await getBalance(context, 2, ALITH))).to.equal(
      BigInt(await getBalance(context, 1, ALITH)) - BigInt(`0x${transferAmount}`)
    );
    expect(BigInt(await getBalance(context, 2, CHARLETH))).to.equal(
      BigInt(await getBalance(context, 1, CHARLETH)) + BigInt(`0x${transferAmount}`)
    );

    const newAllowedAmount = (
      BigInt(`0x${allowedAmount}`) - BigInt(`0x${transferAmount}`)
    ).toString(16);
    await checkAllowance(context, ALITH, BALTATHAR, newAllowedAmount);
  });
});

describeDevMoonbeam("Precompiles - ERC20", (context) => {
  it("refuses to transferFrom more than allowed", async function () {
    const allowedAmount = `1000000000000`.padStart(64, "0");
    const transferAmount = `1400000000000`.padStart(64, "0");

    await sendApprove(context, ALITH, ALITH_PRIV_KEY, BALTATHAR, allowedAmount);

    // transferFrom
    {
      let from = ALITH.slice(2).padStart(64, "0");
      let to = CHARLETH.slice(2).padStart(64, "0");

      let tx = await createTransaction(context.web3, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_ERC20,
        data: `0x${SELECTORS.transferFrom}${from}${to}${transferAmount}`,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(false); // transfer fails because it is not allowed that much
    }

    expect(BigInt(await getBalance(context, 2, ALITH))).to.equal(
      BigInt(await getBalance(context, 1, ALITH))
    );
    expect(BigInt(await getBalance(context, 2, CHARLETH))).to.equal(
      BigInt(await getBalance(context, 1, CHARLETH))
    );

    await checkAllowance(context, ALITH, BALTATHAR, allowedAmount);
  });
});
