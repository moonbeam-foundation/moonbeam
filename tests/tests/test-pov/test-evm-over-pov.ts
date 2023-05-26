import "@moonbeam-network/api-augment";
import Debug from "debug";

import { expect, use as chaiUse } from "chai";
import chaiAsPromised from "chai-as-promised";

import { alith } from "../../util/accounts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { createContract, createTransaction } from "../../util/transactions";
import { MAX_BLOCK_DEV_POV } from "../../util/constants";
import { Contract } from "web3-eth-contract";
const debug = Debug("test:evm-over-pov");

chaiUse(chaiAsPromised);

interface HeavyContract {
  deployed: boolean;
  account: string;
  key: string;
}
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
  }
  return contracts as HeavyContract[];
};

describeDevMoonbeam("PoV Limit (4.2Mb in Dev)", (context) => {
  let contractProxy: Contract;
  let contracts: HeavyContract[];

  before("Deploy the contracts", async function () {
    // Deploy the CallForwarder contract
    const creation = await createContract(context, "CallForwarder");
    contractProxy = creation.contract;
    await context.createBlock(creation.rawTx);

    // Deploy heavy contracts (test won't use more than what is needed for reaching max pov)
    contracts = await deployHeavyContracts(
      context,
      6000,
      6000 + Math.ceil(MAX_BLOCK_DEV_POV / 24_000) + 1
    );
  });

  it("should allow to produce block just under the PoV Limit", async function () {
    const max_contracts = Math.floor(MAX_BLOCK_DEV_POV / 24_000) - 1;

    const { result, block } = await context.createBlock(
      createTransaction(context, {
        to: contractProxy.options.address,
        data: contractProxy.methods
          .callRange(contracts[0].account, contracts[max_contracts].account)
          .encodeABI(),
      })
    );

    debug("block.proof_size", block.proof_size);
    expect(block.proof_size).to.be.at.least(MAX_BLOCK_DEV_POV - 20_000);
    expect(block.proof_size).to.be.at.most(MAX_BLOCK_DEV_POV - 1);
    // The transaction should be not be included in the block
    expect(result.successful).to.equal(true);
  });

  it("should prevent a transaction reaching just over the PoV", async function () {
    const max_contracts = Math.ceil(MAX_BLOCK_DEV_POV / 24_000);

    const { result, block } = await context.createBlock(
      createTransaction(context, {
        to: contractProxy.options.address,
        data: contractProxy.methods
          .callRange(contracts[0].account, contracts[max_contracts].account)
          .encodeABI(),
      })
    );

    debug("block.proof_size", block.proof_size);
    // Empty blocks usually do not exceed 10kb, picking 50kb as a safe limit
    expect(block.proof_size).to.be.at.most(50_000);
    expect(result.successful).to.equal(false);
  });
});
