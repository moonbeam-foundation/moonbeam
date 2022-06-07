import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { describeDevMoonbeamAllEthTxTypes } from "../../util/setup-dev-tests";
import { web3EthCall } from "../../util/providers";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createTransaction,
} from "../../util/transactions";
import { PRECOMPILE_NATIVE_ERC20_ADDRESS } from "../../util/constants";
import { alith, baltathar, charleth } from "../../util/accounts";

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

async function getBalance(context, blockHeight, address) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free;
}

async function sendApprove(context, spender, amount) {
  const fromData = alith.address.slice(2).padStart(64, "0").toLowerCase();
  const spenderData = spender.slice(2).padStart(64, "0").toLowerCase();

  const { result } = await context.createBlockWithEth(
    await createTransaction(context, {
      ...ALITH_TRANSACTION_TEMPLATE,
      to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
      data: `0x${SELECTORS.approve}${spenderData}${amount}`,
    })
  );

  const receipt = await context.web3.eth.getTransactionReceipt(result.result);
  expect(receipt.status).to.equal(true);
  expect(receipt.logs.length).to.eq(1);
  expect(receipt.logs[0].address).to.eq(PRECOMPILE_NATIVE_ERC20_ADDRESS);
  expect(receipt.logs[0].data).to.eq(`0x${amount}`);
  expect(receipt.logs[0].topics.length).to.eq(3);
  expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logApprove);
  expect(receipt.logs[0].topics[1]).to.eq(`0x${fromData}`);
  expect(receipt.logs[0].topics[2]).to.eq(`0x${spenderData}`);
}

async function checkAllowance(context, owner, spender, amount) {
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

    let amount = await getBalance(context, 0, alith.address);
    amount = "0x" + amount.toHex().slice(2).padStart(64, "0");
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

    const { result } = await context.createBlockWithEth(
      await createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
        data: `0x${SELECTORS.transfer}${to}${amount}`,
      })
    );

    const receipt = await context.web3.eth.getTransactionReceipt(result.result);
    expect(receipt.status).to.equal(true);

    const fees = receipt.gasUsed * 1_000_000_000;

    expect(BigInt(await getBalance(context, 1, alith.address))).to.equal(
      BigInt(await getBalance(context, 0, alith.address)) - BigInt(`0x${amount}`) - BigInt(fees)
    );
    expect(BigInt(await getBalance(context, 1, charleth.address))).to.equal(
      BigInt(await getBalance(context, 0, charleth.address)) + BigInt(`0x${amount}`)
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

      const { result } = await context.createBlockWithEth(
        await createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
          data: `0x${SELECTORS.transferFrom}${from}${to}${transferAmount}`,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.result);

      expect(receipt.logs.length).to.eq(1);
      expect(receipt.logs[0].address).to.eq(PRECOMPILE_NATIVE_ERC20_ADDRESS);
      expect(receipt.logs[0].data).to.eq(`0x${transferAmount}`);
      expect(receipt.logs[0].topics.length).to.eq(3);
      expect(receipt.logs[0].topics[0]).to.eq(SELECTORS.logTransfer);
      expect(receipt.logs[0].topics[1]).to.eq(`0x${from}`);
      expect(receipt.logs[0].topics[2]).to.eq(`0x${to}`);

      expect(receipt.status).to.equal(true);
    }

    expect(BigInt(await getBalance(context, 2, alith.address))).to.equal(
      BigInt(await getBalance(context, 1, alith.address)) - BigInt(`0x${transferAmount}`)
    );
    expect(BigInt(await getBalance(context, 2, charleth.address))).to.equal(
      BigInt(await getBalance(context, 1, charleth.address)) + BigInt(`0x${transferAmount}`)
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

      const { result } = await context.createBlockWithEth(
        await createTransaction(context, {
          ...BALTATHAR_TRANSACTION_TEMPLATE,
          to: PRECOMPILE_NATIVE_ERC20_ADDRESS,
          data: `0x${SELECTORS.transferFrom}${from}${to}${transferAmount}`,
        })
      );

      const receipt = await context.web3.eth.getTransactionReceipt(result.result);
      expect(receipt.status).to.equal(false); // transfer fails because it is not allowed that much
    }

    expect(BigInt(await getBalance(context, 2, alith.address))).to.equal(
      BigInt(await getBalance(context, 1, alith.address))
    );
    expect(BigInt(await getBalance(context, 2, charleth.address))).to.equal(
      BigInt(await getBalance(context, 1, charleth.address))
    );

    await checkAllowance(context, alith.address, baltathar.address, allowedAmount);
  });
});
