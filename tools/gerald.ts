import { importAccount, web3, gerald, contractAddress } from "./init-web3";

const main = async () => {
  const nonce = await web3.eth.getTransactionCount(gerald.address);

  const chainId = await web3.eth.net.getId();
  // Step 1: Creating the contract.
  console.log(`Using chain id: ${chainId}\n`);
  console.log(`Gerald account: ${gerald.address} (nonce: ${nonce})`);

  const contractAdd = contractAddress(gerald.address, 0);
  const code = await web3.eth.getCode(contractAdd);
  const storageAdd = await web3.eth.getBalance(contractAdd, "0");
  console.log(`Gerald contract[0]: ${contractAdd} (code: ${code.length} bytes)`);
  console.log(`           storage: ${storageAdd}`);
};

main().catch((err) => {
  console.log("Error", err);
});
