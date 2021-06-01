import { importAccount, compileSolidity, web3, customRequest, init } from "../init-web3";
import { TransactionReceipt } from "web3-core";
import yargs from "yargs";
import * as rlp from "rlp";

const argv = yargs(process.argv.slice(2))
  .usage("Usage: $0  [--net stagenet]")
  .version("1.0.0")
  .options({
    net: { type: "string", default: "stagenet" },
  }).argv;

const contractSource = `
pragma solidity >=0.4.22;

contract Loop {
  function big_loop(uint256 target) pure public returns (uint256) {
    uint256 number = 0;
    for (uint i = 0; i < target; i++) {
      number += 1;
    }
    return number;
  }
}`;

init(
  argv["net"] == "stagenet"
    ? "https://rpc.stagenet.moonbeam.gcp.purestake.run"
    : argv["net"] == "localhost"
    ? "http://127.0.0.1:9933"
    : argv["net"] == "alan"
    ? "http://127.0.0.1:56053"
    : argv["net"] == "alan-standalone"
    ? "http://127.0.0.1:55543"
    : undefined
);

const contractCompiled = compileSolidity(contractSource, "Loop");
const contractBytecode = contractCompiled.bytecode;
const contractAbi = contractCompiled.contract.abi;
const deployer = importAccount("99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342");
const contractAddress = "0x" + web3.utils.sha3(rlp.encode([deployer.address, 0]) as any).substr(26);

let nonce = 0;

const printAddressInfo = async (address: string) => {
  const nonce = await web3.eth.getTransactionCount(address);
  const balance = await web3.eth.getBalance(address);
  console.log(`Account: ${address} (nonce: ${nonce}) => ${balance}`);
};

const deployContract = async () => {
  // 1M gas contract call (big_loop)

  printAddressInfo(deployer.address);

  const code = await customRequest("eth_getCode", [contractAddress]);
  if (code.result != "0x") {
    console.log("Contract already deployed");
    return;
  }

  const tx = await web3.eth.accounts.signTransaction(
    {
      from: deployer.address,
      data: contractBytecode,
      value: "0x00",
      gasPrice: 1_000_000_000,
      gas: 12_995_000,
      nonce: nonce++,
    },
    deployer.privateKey
  );
  const result = await customRequest("eth_sendRawTransaction", [tx.rawTransaction]);
  if (result.error) {
    console.error(`Error deploying contract!`);
    console.error(result.error);
    return;
  }
  console.log(`Transaction sent: ${tx.transactionHash}`);
  const startTime = Date.now();
  while (Date.now() - startTime < 40000) {
    let rcpt: TransactionReceipt = await web3.eth.getTransactionReceipt(tx.transactionHash);
    if (rcpt) {
      console.log(`Transaction done - block #${rcpt.blockNumber} (${rcpt.blockHash})`);
      return;
    }
    await new Promise((resolve) => {
      setTimeout(resolve, 2000);
    });
  }
  throw new Error("Failed to verify contract deployment (timeout)");
};

const callContract = async (loopCount: number) => {
  const contract = new web3.eth.Contract(contractAbi, contractAddress);

  const encoded = await contract.methods.big_loop(loopCount).encodeABI();

  const tx = await web3.eth.accounts.signTransaction(
    {
      from: deployer.address,
      to: contractAddress,
      data: encoded,
      gasPrice: 1_000_000_000,
      gas: 1_200_995_000,
      nonce: nonce++,
    },
    deployer.privateKey
  );

  const result = await customRequest("eth_sendRawTransaction", [tx.rawTransaction]);
  if (result.error) {
    console.error(result.error);
    throw new Error(`Error calling contract!`);
  }

  console.log(`Transaction for Loop count ${loopCount} sent: ${tx.transactionHash}`);
  const startTime = Date.now();
  while (Date.now() - startTime < 60000) {
    let rcpt: TransactionReceipt = await web3.eth.getTransactionReceipt(tx.transactionHash);
    if (rcpt) {
      console.log(`Loop count ${loopCount} - block #${rcpt.blockNumber} (${rcpt.blockHash})`);
      return;
    }
    await new Promise((resolve) => {
      setTimeout(resolve, 2000);
    });
  }
  throw new Error("Failed to verify contract call (timeout)");
};

const main = async () => {
  await deployContract();
  let loopCount = 2;
  while (loopCount > 0) {
    await callContract(loopCount);
    loopCount *= 2;
  }
};

main().catch((err) => {
  console.log("Error", err);
});
