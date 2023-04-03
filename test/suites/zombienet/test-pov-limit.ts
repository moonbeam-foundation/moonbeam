import { describeSuite, expect, beforeAll, ApiPromise } from "@moonwall/cli";
import { alith, ALITH_PRIVATE_KEY, EthTester } from "@moonwall/util";
import { getCompiled } from "../../functions/contract-manager.js";
import Web3 from "web3";

/**
 * @description Deploy multiple contracts to test the EVM storage limit.
 * @param context Context of the test
 * @param count Number of contracts to deploy
 * @returns
 */
const deployHeavyContracts = async (polkadotApi: ApiPromise, first = 6000, last = 6999) => {
  // Generate the contract addresses
  const contracts = await Promise.all(
    new Array(last - first + 1).fill(0).map(async (_, i) => {
      const account = `0x${(i + first).toString(16).padStart(40, "0")}`;
      return {
        deployed: false,
        account,
        key: polkadotApi.query.evm.accountCodes.key(account),
      };
    })
  );

  // Check which contracts are already deployed
  for (const contract of contracts) {
    contract.deployed =
      (await polkadotApi.rpc.state.getStorage(contract.key)).toString().length > 10;
    console.log(contract.deployed, contract.key);
  }

  // Create the contract code (24kb of zeros)
  const evmCode = `60006000fd${"0".repeat(24_000 * 2)}`;
  const storageData = `${polkadotApi.registry
    .createType("Compact<u32>", `0x${BigInt((evmCode.length + 1) * 2).toString(16)}`)
    .toHex(true)}${evmCode}`;

  // Create the batchs of contracts to deploy
  const batchs = contracts
    .reduce(
      (acc, value) => {
        if (acc[acc.length - 1].length >= 40) acc.push([]);
        if (!value.deployed) acc[acc.length - 1].push([value.key, storageData]);
        return acc;
      },
      [[]] as [string, string][][]
    )
    .filter((batch) => batch.length > 0);

  // Set the storage of the contracts
  // let nonce = await polkadotApi.rpc.system.accountNextIndex(alith.address);
  const promises = batchs.map((batch) =>
    polkadotApi.tx.sudo.sudo(polkadotApi.tx.system.setStorage(batch)).signAndSend(alith, {
      nonce: -1,
    })
  );
  await promises;
  return contracts;
};

describeSuite({
  id: "Z1",
  title: "PoV Limit",
  foundationMethods: "zombie",
  testCases: function ({ it, context, log }) {
    let web3: Web3;
    let ethTester: EthTester;

    beforeAll(() => {
      web3 = context.web3();
      ethTester = new EthTester(context.web3(), ALITH_PRIVATE_KEY, log);
    });

    it({
      id: "T01",
      title: "Should revert the transaction if the PoV is too big",
      timeout:600000,
      test: async function () {
        log("Starting:");

        const contract = getCompiled("CallForwarder");
        log("Sending:");
        const response = await ethTester.sendSignedTransaction(
          ethTester.genSignedContractDeployment({
            abi: contract.contract.abi,
            byteCode: contract.byteCode,
          })
        );

        const receipt = await web3.eth.getTransactionReceipt(response.transactionHash);
        log("Received:", JSON.stringify(receipt, null, 2));

        expect(receipt.status).to.equal(false);

        // TODO: add rest of test
      },
    });
  },
});

// describeDevMoonbeam("Reaching PoV Limit", (context) => {
//   // Need to find a way to support PoV/ProofSize limit in the dev service
//   it.skip("should not prevent the node to produce blocks", async function () {
//     this.timeout(60000);

//     console.log("Starting:");
//     // Deploy the CallForwarder contract
//     await context.createBlock(rawTx);
//     console.log("Contract:", contractProxy.options.address);

//     // Deploy the 500 heavy contracts
//     const contracts = await deployHeavyContracts(context, 6000, 6500);

//     // Sends the call generating 20Mb+ of Proof Size
//     const { result } = await context.createBlock(
//       createTransaction(context, {
//         to: contractProxy.options.address,
//         data: contractProxy.methods
//           .callRange(contracts[0].account, contracts[contracts.length - 1].account)
//           .encodeABI(),
//       })
//     );

//     // The transaction should be not be included in the block
//     expect(result.successful).to.equal(false);
//   });
// });

// // describeDevMoonbeam("Estimate Gas - Check Transfer Gas Cost", (context) => {
// //   it("transfer can generate pov", async function () {
// //     this.timeout(10000000);

// //     const contracts = await deployHeavyContracts(context, 200);

// //     let nonce = await context.web3.eth.getTransactionCount(alith.address);

// //     // Deploy the 500 heavy contracts
// //     const accounts = new Array(10)
// //       .fill(0)
// //       .map((_, i) => `0xDEADBEEF${i.toString(16).padStart(56, "0")}`)
// //       .map((privateKey) => ({
// //         privateKey,
// //         address: context.web3.eth.accounts.privateKeyToAccount(privateKey).address,
// //       }));

// //     const { result } = await context.createBlock(
// //       contracts
// //         .filter((_, i) => i < 150)
// //         .map(({ account }) =>
// //           createTransfer(context, account, 1_000_000_000_000_000n, { nonce: nonce++, gas: 300000 })
// //         )
// //     );

// //     console.log(result);
// //     const receipt = await context.web3.eth.getTransactionReceipt(result[0]?.hash);
// //     expect(receipt.gasUsed).to.equal(21006);
// //   });
// // });
