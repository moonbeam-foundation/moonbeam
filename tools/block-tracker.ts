import Web3 from "web3";

const HttpProvider = require("ethjs-provider-http");
const PollingBlockTracker = require("eth-block-tracker");

const provider = new HttpProvider("http://localhost:9933");
const blockTracker = new PollingBlockTracker({ provider, pollingInterval: 500 });
const web3 = new Web3("http://localhost:9933");

let lastTime = Date.now();
const displayBlock = async (blockNumber) => {
  const diffTime = ((Date.now() - lastTime) / 1000).toFixed(1);
  lastTime = Date.now();

  const block = await web3.eth.getBlock(blockNumber);
  console.log(`${blockNumber.toString().padStart(5, " ")}: ${block.hash} (${diffTime} s)`);
  block.transactions.forEach((t: any, index) => {
    console.log(`     [${index}] ${t.hash} (input: ${t.input.length} bytes)`);
    console.log(`     [${index}] from ${t.from} (nonce: ${t.nonce}) to ${t.to}`);
    console.log(`     [${index}] value: ${t.value} gas: ${t.gas} gasPrice: ${t.gasPrice}`);
    if (t.creates) {
      console.log(`     [${index}] creates: ${t.creates}`);
    }
  });
};
blockTracker.on("latest", (data) => displayBlock(parseInt(data, 16)));

// This part requires eth_subscribe

// import Web3 from "web3";

// const web3 = new Web3("ws://localhost:9944");

// var subscription = web3.eth
//   .subscribe("newBlockHeaders", function (error, result) {
//     if (!error) {
//       console.log(result);

//       return;
//     }

//     console.error(error);
//   })
//   .on("data", function (blockHeader) {
//     console.log(blockHeader);
//   })
//   .on("error", console.error);

// // unsubscribes the subscription
// subscription.unsubscribe(function (error, success) {
//   if (success) {
//     console.log("Successfully unsubscribed!");
//   }
// });
