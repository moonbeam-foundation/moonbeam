import { gerald, web3, ERC20_BYTECODE, init } from "./init-web3";
init();

const main = async () => {
  console.log(`\nCreating contract using Eth RPC "sendTransaction" from gerald`);
  const createTransaction = await gerald.signTransaction({
    data: ERC20_BYTECODE,
    value: "0x00",
    gasPrice: "0x00",
    gas: "0x100000",
  });
  console.log("Transaction", {
    ...createTransaction,
    rawTransaction: `${createTransaction.rawTransaction.substring(0, 32)}... (${
      createTransaction.rawTransaction.length
    } length)`,
  });

  const createReceipt = await web3.eth.sendSignedTransaction(createTransaction.rawTransaction);
  console.log(`Contract deployed at address ${createReceipt.contractAddress}`);
};

main().catch((err) => {
  console.log("Error", err);
});
