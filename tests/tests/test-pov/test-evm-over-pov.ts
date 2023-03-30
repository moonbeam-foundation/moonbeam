import "@moonbeam-network/api-augment";

import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";

import { alith } from "../../util/accounts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { createContract, createTransaction } from "../../util/transactions";

chaiUse(chaiAsPromised);

/**
 * @description Deploy multiple contracts to test the EVM storage limit.
 * @param context Context of the test
 * @param count Number of contracts to deploy
 * @returns
 */
const deployHeavyContracts = async (context: DevTestContext, first = 6000, last = 6999) => {
  // Generate the contract addresses
  const contracts = await Promise.all(
    new Array(last - first + 1).fill(0).map(async (_, i) => {
      const account = `0x${(i + first).toString(16).padStart(40, "0")}`;
      return {
        deployed: false,
        account,
        key: context.polkadotApi.query.evm.accountCodes.key(account),
      };
    })
  );

  // Check which contracts are already deployed
  for (const contract of contracts) {
    contract.deployed =
      (await context.polkadotApi.rpc.state.getStorage(contract.key)).toString().length > 10;
    console.log(contract.deployed, contract.key);
  }

  // Create the contract code (24kb of zeros)
  const evmCode = `60006000fd${"0".repeat(24_000 * 2)}`;
  const storageData = `${context.polkadotApi.registry
    .createType("Compact<u32>", `0x${BigInt((evmCode.length + 1) * 2).toString(16)}`)
    .toHex(true)}${evmCode}`;

  // Create the batchs of contracts to deploy
  const batchs = contracts
    .reduce(
      (acc, value) => {
        if (acc[acc.length - 1].length >= 30) acc.push([]);
        if (!value.deployed) acc[acc.length - 1].push([value.key, storageData]);
        return acc;
      },
      [[]] as [string, string][][]
    )
    .filter((batch) => batch.length > 0);

  // Set the storage of the contracts
  let nonce = await context.web3.eth.getTransactionCount(alith.address);
  for (let i = 0; i < batchs.length; i++) {
    const batch = batchs[i];
    await context.createBlock([
      context.polkadotApi.tx.sudo
        .sudo(context.polkadotApi.tx.system.setStorage(batch))
        .signAsync(alith, {
          nonce: nonce++,
        }),
    ]);
    console.log("batch:", i, batch.length);
  }
  return contracts;
};

describeDevMoonbeam("Reaching PoV Limit", (context) => {
  // Need to find a way to support PoV/ProofSize limit in the dev service
  it.skip("should not prevent the node to produce blocks", async function () {
    this.timeout(60000);

    console.log("Starting:");
    // Deploy the CallForwarder contract
    const { contract: contractProxy, rawTx } = await createContract(context, "CallForwarder");
    await context.createBlock(rawTx);
    console.log("Contract:", contractProxy.options.address);

    // Deploy the 500 heavy contracts
    const contracts = await deployHeavyContracts(context, 6000, 6500);

    // Sends the call generating 20Mb+ of Proof Size
    const { result } = await context.createBlock(
      createTransaction(context, {
        to: contractProxy.options.address,
        data: contractProxy.methods
          .callRange(contracts[0].account, contracts[contracts.length - 1].account)
          .encodeABI(),
      })
    );

    // The transaction should be not be included in the block
    expect(result.successful).to.equal(false);
  });
});

// describeDevMoonbeam("Estimate Gas - Check Transfer Gas Cost", (context) => {
//   it("transfer can generate pov", async function () {
//     this.timeout(10000000);

//     const contracts = await deployHeavyContracts(context, 200);

//     let nonce = await context.web3.eth.getTransactionCount(alith.address);

//     // Deploy the 500 heavy contracts
//     const accounts = new Array(10)
//       .fill(0)
//       .map((_, i) => `0xDEADBEEF${i.toString(16).padStart(56, "0")}`)
//       .map((privateKey) => ({
//         privateKey,
//         address: context.web3.eth.accounts.privateKeyToAccount(privateKey).address,
//       }));

//     const { result } = await context.createBlock(
//       contracts
//         .filter((_, i) => i < 150)
//         .map(({ account }) =>
//           createTransfer(context, account, 1_000_000_000_000_000n, { nonce: nonce++, gas: 300000 })
//         )
//     );

//     console.log(result);
//     const receipt = await context.web3.eth.getTransactionReceipt(result[0]?.hash);
//     expect(receipt.gasUsed).to.equal(21006);
//   });
// });
