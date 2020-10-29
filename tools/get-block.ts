import Web3 from "web3";
const web3 = new Web3("http://localhost:9933");

let lastTime = Date.now();
const displayBlock = async (blockNumber) => {
  const diffTime = ((Date.now() - lastTime) / 1000).toFixed(1);
  lastTime = Date.now();

  const block = await web3.eth.getBlock(blockNumber);
  console.log(`${blockNumber.toString().padStart(5, " ")}: ${block.hash} (${diffTime} s)`);
  block.transactions.forEach((t: any, index) => {
    console.log(`     [${index}] ${t.hash} (input: ${t.input && t.input.length} bytes)`);
    console.log(`     [${index}] from ${t.from} (nonce: ${t.nonce}) to ${t.to}`);
    console.log(`     [${index}] value: ${t.value} gas: ${t.gas} gasPrice: ${t.gasPrice}`);
    if (t.creates) {
      console.log(`     [${index}] creates: ${t.creates}`);
    }
  });
};

var myArgs = process.argv.slice(2);
displayBlock(myArgs[0]);
