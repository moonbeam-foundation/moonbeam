import "@moonbeam-network/api-augment";

import { expect } from "chai";

import { alith, baltathar, charleth } from "../../util/accounts";
import { PRECOMPILE_NATIVE_ERC20_ADDRESS } from "../../util/constants";
import { web3EthCall } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createTransaction,
} from "../../util/transactions";

const SELECTORS = {
  balanceOf: "70a08231",
  totalSupply: "18160ddd",
  approve: "095ea7b3",
  allowance: "dd62ed3e",
  transfer: "a9059cbb",
  transferFrom: "23b872dd",
  logApprove: "0x8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b925",
  logTransfer: "0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef",
};

async function getBalance(context: DevTestContext, blockHeight: number, address: string) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free;
}

async function sendApprove(context: DevTestContext, spender: string, amount: string) {
  const fromData = alith.address.slice(2).padStart(64, "0").toLowerCase();
  const spenderData = spender.slice(2).padStart(64, "0").toLowerCase();

  const { result } = await context.createBlock(
    createTransaction(context, {
      ...ALITH_TRANSACTION_TEMPLATE,
      to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
      data: `0x${SELECTORS.approve}${spenderData}${amount}`,
    })
  );

  const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
  expect(receipt.status).to.equal(true);
  expect(receipt.logs.length).to.eq(1);
  expect(receipt.logs[0].address).to.eq(PRECOMPILE_NATIVE_ERC20_ADDRESS);
  expect(receipt.logs[0].data).to.eq(`0x${amount}`);
  expect(receipt.logs[0].topics.length).to.eq(3);
  expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
  expect(receipt.logs[0].topics[1]).to.eq(`0x${fromData}`);
  expect(receipt.logs[0].topics[2]).to.eq(`0x${spenderData}`);
}

async function checkAllowance(
  context: DevTestContext,
  owner: string,
  spender: string,
  amount: string
) {
  const ownerData = owner.slice(2).padStart(64, "0");
  const spenderData = spender.slice(2).padStart(64, "0");

  const request = await web3EthCall(context.web3, {
    to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
    data: `0x${SELECTORS.allowance}${ownerData}${spenderData}`,
  });

  expect(request.result).equals(`0x${amount.padStart(64, "0")}`);
}

describeDevMoonbeamAllEthTxTypes("Precompiles - ERC20 Native", (context) => {
  it("allows to call getBalance", async function () {
    const address = alith.address.slice(2).padStart(64, "0");

    const request = await web3EthCall(context.web3, {
      to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
      data: `0x${SELECTORS.balanceOf}${address}`,
    });

    const amount =
      "0x" + (await getBalance(context, 0, alith.address)).toHex().slice(2).padStart(64, "0");
    expect(request.result).equals(amount);
  });

  it("allows to call totalSupply", async function () {
    const request = await web3EthCall(context.web3, {
      to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
      data: `0x${SELECTORS.totalSupply}`,
    });

    const amount = await context.polkadotApi.query.balances.totalIssuance();
    const amount_hex = "0x" + amount.toHex().slice(2).padStart(64, "0");

    expect(request.result).equals(amount_hex);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - ERC20 Native", (context) => {
  it("allows to approve transfers, and allowance matches", async function () {
    const amount = `1000000000000`.padStart(64, "0");
    await sendApprove(context, baltathar.address, amount);
    await checkAllowance(context, alith.address, baltathar.address, amount);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - ERC20 Native", (context) => {
  it("allows to call transfer", async function () {
    const amount = `400000000000`.padStart(64, "0");

    const to = charleth.address.slice(2).padStart(64, "0");

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
        data: `0x${SELECTORS.transfer}${to}${amount}`,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    const fees = receipt.gasUsed * 1_000_000_000;

    expect((await getBalance(context, 1, alith.address)).toBigInt()).to.equal(
      (await getBalance(context, 0, alith.address)).toBigInt() -
        BigInt(`0x${amount}`) -
        BigInt(fees)
    );
    expect((await getBalance(context, 1, charleth.address)).toBigInt()).to.equal(
      (await getBalance(context, 0, charleth.address)).toBigInt() + BigInt(`0x${amount}`)
    );
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - ERC20 Native", (context) => {
  it("allows to approve transfer and use transferFrom", async function () {
    const allowedAmount = `1000000000000`.padStart(64, "0");
    const transferAmount = `400000000000`.padStart(64, "0");

    await sendApprove(context, baltathar.address, allowedAmount);

    // transferFrom
    {
      const from = alith.address.slice(2).padStart(64, "0").toLowerCase();
      const to = charleth.address.slice(2).padStart(64, "0").toLowerCase();

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
          data: `0x${SELECTORS.transferFrom}${from}${to}${transferAmount}`,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);

      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address).to.eq(PRECOMPILE_NATIVE_ERC20_ADDRESS);
      expect(receipt.logs[0].data).to.eq(`0x${transferAmount}`);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
      expect(receipt.logs[0].topics[1]).to.eq(`0x${from}`);
      expect(receipt.logs[0].topics[2]).to.eq(`0x${to}`);

      expect(receipt.status).to.equal(true);
    }

    expect((await getBalance(context, 2, alith.address)).toBigInt()).to.equal(
      (await getBalance(context, 1, alith.address)).toBigInt() - BigInt(`0x${transferAmount}`)
    );
    expect((await getBalance(context, 2, charleth.address)).toBigInt()).to.equal(
      (await getBalance(context, 1, charleth.address)).toBigInt() + BigInt(`0x${transferAmount}`)
    );

    const newAllowedAmount = (
      BigInt(`0x${allowedAmount}`) - BigInt(`0x${transferAmount}`)
    ).toString(16);
    await checkAllowance(context, alith.address, baltathar.address, newAllowedAmount);
  });
});

describeDevMoonbeamAllEthTxTypes("Precompiles - ERC20", (context) => {
  it("refuses to transferFrom more than allowed", async function () {
    const allowedAmount = `1000000000000`.padStart(64, "0");
    const transferAmount = `1400000000000`.padStart(64, "0");

    await sendApprove(context, baltathar.address, allowedAmount);

    // transferFrom
    {
      let from = alith.address.slice(2).padStart(64, "0");
      let to = charleth.address.slice(2).padStart(64, "0");

      const { result } = await context.createBlock(
        createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
          data: `0x${SELECTORS.transferFrom}${from}${to}${transferAmount}`,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
      expect(receipt.status).to.equal(false); // transfer fails because it is not allowed that much
    }

    expect((await getBalance(context, 2, alith.address)).toBigInt()).to.equal(
      (await getBalance(context, 1, alith.address)).toBigInt()
    );
    expect((await getBalance(context, 2, charleth.address)).toBigInt()).to.equal(
      (await getBalance(context, 1, charleth.address)).toBigInt()
    );

    await checkAllowance(context, alith.address, baltathar.address, allowedAmount);
  });
});
