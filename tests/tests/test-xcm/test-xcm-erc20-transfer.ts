import "@moonbeam-network/api-augment";
import { bnToHex } from "@polkadot/util";

import { expect } from "chai";
import { ethers } from "ethers";
import { Contract } from "web3-eth-contract";

import { ALITH_ADDRESS, BALTATHAR_ADDRESS } from "../../util/accounts";
import { PRECOMPILE_XTOKENS_ADDRESS } from "../../util/constants";
import { web3EthCall } from "../../util/providers";
import { getCompiled } from "../../util/contracts";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  ALITH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
} from "../../util/transactions";
import { expectEVMResult } from "../../util/eth-transactions";

const ERC20_CONTRACT = getCompiled("ERC20WithInitialSupply");
const ERC20_INTERFACE = new ethers.utils.Interface(ERC20_CONTRACT.contract.abi);
const ERC20_TOTAL_SUPPLY = 1_000_000_000n;
const XTOKENS_CONTRACT = getCompiled("XtokensInstance");
const XTOKENS_INTERFACE = new ethers.utils.Interface(XTOKENS_CONTRACT.contract.abi);

async function getBalance(context: DevTestContext, blockHeight: number, address: string) {
  const blockHash = await context.polkadotApi.rpc.chain.getBlockHash(blockHeight);
  const account = await context.polkadotApi.query.system.account.at(blockHash, address);
  return account.data.free.toBigInt();
}

const setupErc20Contract = async (context: DevTestContext) => {
  const { contract, contractAddress, rawTx } = await createContract(
    context,
    "ERC20WithInitialSupply",
    {
      ...ALITH_TRANSACTION_TEMPLATE,
      gas: 5_000_000,
    },
    ["MyToken", "TKN", ALITH_ADDRESS, ERC20_TOTAL_SUPPLY]
  );
  const { result } = await context.createBlock(rawTx);
  expectEVMResult(result.events, "Succeed");
  return { contract, contractAddress };
};

describeDevMoonbeam("Mock XCM - receive downward transfer", (context) => {
  let erc20Contract: Contract;
  let erc20ContractAddress: string;

  before("Should Register an asset and set unit per sec", async function () {
    const { contract, contractAddress } = await setupErc20Contract(context);
    erc20Contract = contract;
    erc20ContractAddress = contractAddress;
  });

  it("Should be able to transfer ERC20 token throught xcm with xtokens precomp", async function () {
    const amountTransferred = 1000n;
    // Destination as multilocation
    const destination = [
      // one parent
      1,
      // This represents X1(AccountKey20(BALTATHAR_ADDRESS, NetworkAny))
      // AccountKey20 variant (03) + the 20 bytes account + Any network variant (00)
      ["0x03" + BALTATHAR_ADDRESS.slice(2) + "00"],
    ];
    const data = XTOKENS_INTERFACE.encodeFunctionData(
      // action
      "transfer",
      [
        // address of the multiasset
        erc20ContractAddress,
        // amount
        amountTransferred,
        // Destination as multilocation
        destination,
        // weight
        500_000_000n,
      ]
    );

    const { result } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_XTOKENS_ADDRESS,
        data,
      })
    );
    expectEVMResult(result.events, "Succeed");

    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    const gasPrice = receipt.effectiveGasPrice;
    const fees = BigInt(receipt.gasUsed) * BigInt(gasPrice);

    // Fees should have been spent
    expect(await getBalance(context, 2, ALITH_ADDRESS)).to.equal(
      (await getBalance(context, 1, ALITH_ADDRESS)) - fees
    );

    // Erc20 tokens should have been spent
    expect(
      (
        await web3EthCall(context.web3, {
          to: erc20ContractAddress,
          data: ERC20_INTERFACE.encodeFunctionData("balanceOf", [ALITH_ADDRESS]),
        })
      ).result
    ).equals(bnToHex(ERC20_TOTAL_SUPPLY - amountTransferred, { bitLength: 256 }));
  });
});
