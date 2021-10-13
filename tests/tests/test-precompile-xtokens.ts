import { expect } from "chai";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { customWeb3Request } from "../util/providers";
import { ethers } from "ethers";
import { getCompiled } from "../util/contracts";

import { GENESIS_ACCOUNT } from "../util/constants";

const ADDRESS_XTOKENS = "0x0000000000000000000000000000000000000804";
const BALANCES_ADDRESS = "0x0000000000000000000000000000000000000802";

const GAS_PRICE = "0x" + (1_000_000_000).toString(16);

describeDevMoonbeam("Precompiles - xtokens", (context) => {
  let iFace;
  let contractData; 

  before("Deploy contract", async () => {
    contractData = await getCompiled("XtokensInstance");
    iFace = new ethers.utils.Interface(contractData.contract.abi);
  });

  it("allows to issue transfer xtokens", async function () {
    const data = iFace.encodeFunctionData(
      // action
      "transfer",
      [
        // address of the multiasset, in this case our own balances
        BALANCES_ADDRESS,
        // amount
        100,
        // Destination as multilocation
        [
          // one parent
          1,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          ["0x01010101010101010101010101010101010101010101010101010101010101010100"],
        ],
        // weight
        100,
      ]
    );

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_XTOKENS,
        data: data,
      },
    ]);
    // result only exists if the call is succesful
    expect(tx_call.hasOwnProperty("result"));
  });

  it("allows to issue transfer_multiasset xtokens", async function () {
    const data = iFace.encodeFunctionData(
      // action
      "transfer_multiasset",
      [
        // Asset as MultiLocation + amount
        [
          // one parent
          1,
          // X2(Parachain, PalletInstance)
          // Parachain: Parachain selector (00) + parachain id (1000) in 4 bytes (000003E8)
          // PalletInstance: Selector (04) + pallet instance 1 byte (03)
          ["0x00000003e8", "0x0403"],
        ],
        // amount
        100,
        // Destination as multilocation
        [
          // one parent
          1,
          // junction: AccountId32 enum (01) + the 32 byte account + Any network selector(00)
          ["0x01010101010101010101010101010101010101010101010101010101010101010100"],
        ],
        // weight
        100,
      ]
    );

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: GAS_PRICE,
        to: ADDRESS_XTOKENS,
        data: data,
      },
    ]);
    // result only exists if the call is succesful
    expect(tx_call.hasOwnProperty("result"));
  });
});
