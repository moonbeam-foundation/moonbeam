import "@moonbeam-network/api-augment";

import { expect } from "chai";
import { ethers } from "ethers";

import { alith, ALITH_PRIVATE_KEY } from "../../util/accounts";
import { getCompiled } from "../../util/contracts";
import { customWeb3Request } from "../../util/providers";
import { describeDevMoonbeamAllEthTxTypes, DevTestContext } from "../../util/setup-dev-tests";

// We use ethers library in this test as apparently web3js's types are not fully EIP-1559
// compliant yet.
describeDevMoonbeamAllEthTxTypes("Fee History", (context) => {
  async function sendTransaction(context: DevTestContext, payload: any) {
    let signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);
    // Ethers internally matches the locally calculated transaction hash against the one
    // returned as a response.
    // Test would fail in case of mismatch.
    const tx = await signer.sendTransaction(payload);
    return tx;
  }

  function get_percentile(percentile: number, array: number[]) {
    array.sort(function (a, b) {
      return a - b;
    });
    let index = (percentile / 100) * array.length - 1;
    if (Math.floor(index) == index) {
      return array[index];
    } else {
      return Math.ceil((array[Math.floor(index)] + array[Math.ceil(index)]) / 2);
    }
  }

  async function createBlocks(
    block_count: number,
    reward_percentiles: number[],
    priority_fees: number[],
    max_fee_per_gas: string
  ) {
    const contractData = await getCompiled("MultiplyBy7");
    let nonce = await context.web3.eth.getTransactionCount(alith.address);
    for (var b = 0; b < block_count; b++) {
      for (var p = 0; p < priority_fees.length; p++) {
        await sendTransaction(context, {
          from: alith.address,
          data: contractData.byteCode,
          value: "0x00",
          maxFeePerGas: max_fee_per_gas,
          maxPriorityFeePerGas: context.web3.utils.numberToHex(priority_fees[p]),
          accessList: [],
          nonce: nonce,
          gasLimit: "0x100000",
          chainId: 1281,
        });
        nonce++;
      }
      await context.createBlock();
    }
  }

  it("result length should match spec", async function () {
    this.timeout(100000);
    let max_fee_per_gas = "0x3B9ACA00";
    let block_count = 2;
    let reward_percentiles = [20, 50, 70];
    let priority_fees = [1, 2, 3];
    await createBlocks(block_count, reward_percentiles, priority_fees, max_fee_per_gas);
    let result = (
      await customWeb3Request(context.web3, "eth_feeHistory", ["0x2", "latest", reward_percentiles])
    ).result;

    // baseFeePerGas is always the requested block range + 1 (the next derived base fee).
    expect(result.baseFeePerGas.length).to.be.eq(block_count + 1);
    // gasUsedRatio for the requested block range.
    expect(result.gasUsedRatio.length).to.be.eq(block_count);
    // two-dimensional reward list for the requested block range.
    expect(result.reward.length).to.be.eq(block_count);
    // each block has a reward list which's size is the requested percentile list.
    for (let i = 0; i < block_count; i++) {
      expect(result.reward[i].length).to.be.eq(reward_percentiles.length);
    }
  });

  it("should calculate percentiles", async function () {
    this.timeout(100000);
    let max_fee_per_gas = "0x3B9ACA00";
    let block_count = 11;
    let reward_percentiles = [20, 50, 70, 85, 100];
    let priority_fees = [1, 2, 3, 4, 5, 6, 7, 8, 9, 10];
    await createBlocks(block_count, reward_percentiles, priority_fees, max_fee_per_gas);
    let result = (
      await customWeb3Request(context.web3, "eth_feeHistory", ["0xA", "latest", reward_percentiles])
    ).result;

    // Calculate the percentiles in javascript.
    let local_rewards = [];
    for (let i = 0; i < reward_percentiles.length; i++) {
      local_rewards.push(get_percentile(reward_percentiles[i], priority_fees));
    }
    // Compare the rpc result with the javascript percentiles.
    for (let i = 0; i < result.reward.length; i++) {
      // We only test if BaseFee update is enabled.
      //
      // If BaseFee is a constant 1GWEI, that means that there is no effective reward using the
      // specs formula MIN(tx.maxPriorityFeePerGas, tx.maxFeePerGas-block.baseFee).
      //
      // In other words, for this tip oracle there would be no need to provide a priority fee
      // when the block fullness is considered ideal in an EIP-1559 chain.
      if (
        context.web3.utils.hexToNumber(max_fee_per_gas) -
          context.web3.utils.hexToNumber(result.baseFeePerGas[i]) >
        0
      ) {
        expect(result.reward[i].length).to.be.eq(local_rewards.length);
        for (let j = 0; j < local_rewards.length; j++) {
          expect(context.web3.utils.hexToNumber(result.reward[i][j])).to.be.eq(local_rewards[j]);
        }
      }
    }
  });
});
