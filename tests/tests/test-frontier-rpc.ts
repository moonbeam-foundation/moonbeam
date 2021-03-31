import { step } from "mocha-steps";
import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";
import {
  GENESIS_ACCOUNT,
  GENESIS_ACCOUNT_PRIVATE_KEY,
  MANUAL_STORAGE_ABI,
  MANUAL_STORAGE_BYTECODE,
} from "./constants";
import { expect } from "chai";

describeWithMoonbeam("Frontier RPC", `simple-specs.json`, (context) => {
  step("eth_getStorageAt should work", async function () {
    const contract = new context.web3.eth.Contract(MANUAL_STORAGE_ABI);

    const tx0 = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: MANUAL_STORAGE_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x500000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    await customRequest(context.web3, "eth_sendRawTransaction", [tx0.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    let receipt0 = await context.web3.eth.getTransactionReceipt(tx0.transactionHash);

    let contractAddress = receipt0.contractAddress;

    let getStorage0 = await customRequest(context.web3, "eth_getStorageAt", [
      contractAddress,
      "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc",
      "latest",
    ]);

    expect(getStorage0.result).to.be.eq(
      "0x0000000000000000000000000000000000000000000000000000000000000000"
    );

    const tx1 = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        to: contractAddress,
        data: contract.methods
          .setStorage(
            "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc",
            "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
          )
          .encodeABI(),
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x500000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    await customRequest(context.web3, "eth_sendRawTransaction", [tx1.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    let receip1 = await context.web3.eth.getTransactionReceipt(tx1.transactionHash);

    let getStorage1 = await customRequest(context.web3, "eth_getStorageAt", [
      contractAddress,
      "0x360894a13ba1a3210667c828492db98dca3e2076cc3735a920a3ca505d382bbc",
      "latest",
    ]);

    expect(getStorage1.result).to.be.eq(
      "0x0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef"
    );
  });
});
