import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
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

describeDevMoonbeam("Precompiles - ERC20", (context) => {
  it("ERC20 Native currency - getBalance", async function () {
    let selector = `70a08231`; // balanceOf
    let address = ALITH.slice(2).padStart(64, "0");

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000801",
        data: `0x${selector}${address}`,
      },
    ]);

    const genesisHash = await context.polkadotApi.rpc.chain.getBlockHash(0);
    const account = await context.polkadotApi.query.system.account.at(genesisHash, ALITH);
    let amount = "0x" + account.data.free.toHex().slice(2).padStart(64, "0");

    expect(tx_call.result).equals(amount);
  });

  it("ERC20 Native currency - total issuance", async function () {
    let selector = `7c80aa9f`; // totalSupply

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000801",
        data: `0x${selector}`,
      },
    ]);

    let amount = await context.polkadotApi.query.balances.totalIssuance();
    let amount_hex = "0x" + amount.toHex().slice(2).padStart(64, "0");

    expect(tx_call.result).equals(amount_hex);
  });
});

describeDevMoonbeam("Precompiles - ERC20", (context) => {
  it("ERC20 Native currency - approve + allowance", async function () {
    let amount = `1000000000000`.padStart(64, "0");

    // approve
    {
      let selector = `095ea7b3`; // approve
      let spender = BALTATHAR.slice(2).padStart(64, "0");

      let tx = await createTransaction(context.web3, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000801",
        data: `0x${selector}${spender}${amount}`,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(true);
    }

    // allowance
    {
      let selector = `dd62ed3e`; // allowance
      let owner = ALITH.slice(2).padStart(64, "0");
      let spender = BALTATHAR.slice(2).padStart(64, "0");

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: GENESIS_ACCOUNT,
          value: "0x0",
          gas: "0x10000",
          gasPrice: context.web3.utils.numberToHex(1_000_000_000),
          to: "0x0000000000000000000000000000000000000801",
          data: `0x${selector}${owner}${spender}`,
        },
      ]);

      expect(tx_call.result).equals(`0x${amount}`);
    }
  });
});

describeDevMoonbeam("Precompiles - ERC20", (context) => {
  it("ERC20 Native currency - approve + transfer_from", async function () {
    let initialAllowed = `1000000000000`.padStart(64, "0");

    // approve
    {
      let selector = `095ea7b3`; // approve
      let spender = BALTATHAR.slice(2).padStart(64, "0");

      let tx = await createTransaction(context.web3, {
        from: ALITH,
        privateKey: ALITH_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000801",
        data: `0x${selector}${spender}${initialAllowed}`,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(true);
    }

    let amount = `400000000000`.padStart(64, "0");

    let aliceBalancePre;
    let charlethBalancePre;

    {
      const genesisHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
      const account = await context.polkadotApi.query.system.account.at(genesisHash, ALITH);
      aliceBalancePre = account.data.free;
    }

    {
      const genesisHash = await context.polkadotApi.rpc.chain.getBlockHash(1);
      const account = await context.polkadotApi.query.system.account.at(genesisHash, CHARLETH);
      charlethBalancePre = account.data.free;
    }

    // transferFrom
    {
      let selector = `0c41b033`; // transferFrom
      let from = ALITH.slice(2).padStart(64, "0");
      let to = CHARLETH.slice(2).padStart(64, "0");

      let tx = await createTransaction(context.web3, {
        from: BALTATHAR,
        privateKey: BALTATHAR_PRIV_KEY,
        value: "0x0",
        gas: "0x200000",
        gasPrice: context.web3.utils.numberToHex(1_000_000_000),
        to: "0x0000000000000000000000000000000000000801",
        data: `0x${selector}${from}${to}${amount}`,
      });

      let block = await context.createBlock({
        transactions: [tx],
      });

      const receipt = await context.web3.eth.getTransactionReceipt(block.txResults[0].result);
      expect(receipt.status).to.equal(true);
    }

    let aliceBalancePost;
    let charlethBalancePost;

    {
      const b0Hash = await context.polkadotApi.rpc.chain.getBlockHash(2);
      const account = await context.polkadotApi.query.system.account.at(b0Hash, ALITH);
      aliceBalancePost = account.data.free;
    }

    {
      const b0Hash = await context.polkadotApi.rpc.chain.getBlockHash(2);
      const account = await context.polkadotApi.query.system.account.at(b0Hash, CHARLETH);
      charlethBalancePost = account.data.free;
    }

    expect(BigInt(aliceBalancePost)).to.equal(BigInt(aliceBalancePre) - BigInt(`0x${amount}`));
    expect(BigInt(charlethBalancePost)).to.equal(
      BigInt(charlethBalancePre) + BigInt(`0x${amount}`)
    );

    // allowance
    {
      let selector = `dd62ed3e`; // allowance
      let owner = ALITH.slice(2).padStart(64, "0");
      let spender = BALTATHAR.slice(2).padStart(64, "0");

      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: GENESIS_ACCOUNT,
          value: "0x0",
          gas: "0x10000",
          gasPrice: context.web3.utils.numberToHex(1_000_000_000),
          to: "0x0000000000000000000000000000000000000801",
          data: `0x${selector}${owner}${spender}`,
        },
      ]);

      // check that allowance has been decreased by the correct amount.
      expect(tx_call.result.slice(2)).equals(
        (BigInt(`0x${initialAllowed}`) - BigInt(`0x${amount}`)).toString(16).padStart(64, "0")
      );
    }
  });
});
