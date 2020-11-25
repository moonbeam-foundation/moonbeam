
// +++ TransactionConfig +++

import { GENESIS_ACCOUNT, TEST_ACCOUNT, TEST_CONTRACT_BYTECODE } from ".";

import { TransactionConfig } from "web3-core";

export const basicTransfertx: TransactionConfig = {
    from: GENESIS_ACCOUNT,
    to: TEST_ACCOUNT,
    value: "0x200", // =512 Must me higher than ExistentialDeposit (500)
    gasPrice: "0x01",
    gas: "0x100000",
  };
  export const contractCreation: TransactionConfig = {
    from: GENESIS_ACCOUNT,
    data: TEST_CONTRACT_BYTECODE,
    value: "0x00",
    gasPrice: "0x01",
    gas: "0x100000",
  };