  import {SignedTransaction, TransactionConfig} from 'web3-core'
  
  // Test variables 
  export const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  export const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";
    export const TEST_ACCOUNT = "0x1111111111111111111111111111111111111111";
    export const basicTransfertx: TransactionConfig={
      from: GENESIS_ACCOUNT,
      to: TEST_ACCOUNT,
      value: "0x200", // Must me higher than ExistentialDeposit (500)
      gasPrice: "0x01",
      gas: "0x100000",
    }