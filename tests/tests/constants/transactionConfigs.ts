// +++ TransactionConfig +++

import { GENESIS_ACCOUNT, TEST_ACCOUNT } from "./constants";

import { TransactionConfig } from "web3-core";
import { getCompiled } from "../util/contracts";

export type CompleteTransactionConfig =
  | TransactionConfig
  | {
      chainId?: string;
    };

export const basicTransfertx: CompleteTransactionConfig = {
  from: GENESIS_ACCOUNT,
  to: TEST_ACCOUNT,
  value: "0x200", // =512 Must me higher than ExistentialDeposit (500)
  gasPrice: "0x01",
  gas: 21000,
  chainId: "0x501", // Prevents web3 from requesting the chainId
};

export async function contractCreation(): Promise<CompleteTransactionConfig> {
  return {
    from: GENESIS_ACCOUNT,
    data: (await getCompiled("TestContract")).byteCode,
    value: "0x00",
    gasPrice: "0x01",
    gas: 91019,
    chainId: "0x501", // Prevents web3 from requesting the chainId
  };
}
