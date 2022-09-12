import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { ALITH_TRANSACTION_TEMPLATE, createTransaction } from "../../util/transactions";
import { web3EthCall } from "../../util/providers";
import { GLMR, PRECOMPILE_DISPATCH_ADDRESS } from "../../util/constants";
import { ALITH_ADDRESS } from "../../util/accounts";

describeDevMoonbeam("Precompile - Dispatch - fails on pallet-ethereum", (context) => {
  // TODO: rework this test, dispatch precompile disabled
  it.skip("should prevent dispatch precompile on pallet-ethereum", async function () {
    let randomAddress = "0x1111111111111111111111111111111111111111";
    // Signature is verified prior to the execution and not in the pallet
    // Hence even mock signature should be accepted if the execution was
    // possible
    let transaction = {
      EIP1559: {
        chainId: 1281,
        nonce: 0,
        maxPriorityFeePerGas: 0,
        maxFeePerGas: 1000000000,
        gasLimit: 500_000n,
        action: {
          call: randomAddress,
        },
        value: 1n * GLMR,
        input: "0x",
        accessList: [],
        oddYParity: false,
        r: "0xff6a476d2d8d7b0a23fcb3f1471d1ddd4dec7f3803db7f769aa1ce2575e493ac",
        s: "0x4ebec202edd10edfcee87927090105b95d8b58f80550cdf4eda20327f3377ca6",
      },
    };

    let randomBefore = await context.web3.eth.getBalance(randomAddress);

    let ethereumCall = context.polkadotApi.tx.ethereum.transact(transaction);
    let callBytes = ethereumCall?.method.toHex() || "";

    // We first try with call, to see the error message
    let result = await web3EthCall(context.web3, {
      from: ALITH_ADDRESS,
      to: PRECOMPILE_DISPATCH_ADDRESS,
      data: callBytes,
    });

    // Unfortunately, the precompile is not very precise on the error
    expect((result.error as any).message.includes("dispatch execution failed")).to.be.true;

    // Then we insert it in real block
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_DISPATCH_ADDRESS,
        data: callBytes,
      })
    );
    let randomAfter = await context.web3.eth.getBalance(randomAddress);

    // Random address did not receive money
    expect(randomBefore).to.eq(randomAfter);
  });
});

describeDevMoonbeam("Precompile - Dispatch - fails in pallet-ethereum-xcm", (context) => {
  // TODO: rework this test, dispatch precompile disabled
  it.skip("should prevent dispatch precompile on pallet-ethereum-xcm", async function () {
    let randomAddress = "0x1111111111111111111111111111111111111111";

    let transaction = {
      V2: {
        gas_limit: 500_000n,
        action: {
          Call: "0x1111111111111111111111111111111111111111",
        },
        value: 1n * GLMR,
        input: [],
        access_list: null,
      },
    };

    let randomBefore = await context.web3.eth.getBalance(randomAddress);

    let ethereumCall = context.polkadotApi.tx.ethereumXcm.transact(transaction as any);
    let callBytes = ethereumCall?.method.toHex() || "";

    // We first try with call, to see the error message
    let result = await web3EthCall(context.web3, {
      from: ALITH_ADDRESS,
      to: PRECOMPILE_DISPATCH_ADDRESS,
      data: callBytes,
    });

    expect((result.error as any).message.includes("dispatch execution failed")).to.be.true;

    // Then we insert it in real block
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_DISPATCH_ADDRESS,
        data: callBytes,
      })
    );
    let randomAfter = await context.web3.eth.getBalance(randomAddress);

    // Random address did not receive money
    expect(randomBefore).to.eq(randomAfter);
  });
});

describeDevMoonbeam("Precompile - Dispatch - works in pallet-balances", (context) => {
  // TODO: rework this test, dispatch precompile disabled
  it.skip("should allow dispatches for regular pallets (e.g., balances)", async function () {
    let randomAddress = "0x1111111111111111111111111111111111111111";
    let amountToTransfer = 1n * GLMR;
    let balancesCall = context.polkadotApi.tx.balances.transfer(randomAddress, amountToTransfer);
    let callBytes = balancesCall?.method.toHex() || "";

    // We first try with call, to see the error message
    let result = await web3EthCall(context.web3, {
      from: ALITH_ADDRESS,
      to: PRECOMPILE_DISPATCH_ADDRESS,
      data: callBytes,
    });

    // No error this time
    expect(result.error).to.be.eq(undefined);

    // Then we insert it in real block
    await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_DISPATCH_ADDRESS,
        data: callBytes,
      })
    );
    let randomAfter = await context.web3.eth.getBalance(randomAddress);

    // Random address did receive the money this time
    expect(randomAfter.toString()).to.eq(amountToTransfer.toString());
  });
});
