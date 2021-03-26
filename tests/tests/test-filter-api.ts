import { expect } from "chai";
import { step } from "mocha-steps";
import { create } from "ts-node";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

describeWithMoonbeam("Moonbeam RPC (EthFilterApi)", `simple-specs.json`, (context) => {
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  // This reflects the measured gas cost of the transaction at this current point in time.
  // It has been known to fluctuate from release to release, so it may need adjustment.
  const EXPECTED_TRANSACTION_GAS_COST = 891328;

  const TEST_CONTRACT_BYTECODE =
    "0x608060405234801561001057600080fd5b50610041337fffffffffffffffffffffffffffffffffff" +
    "ffffffffffffffffffffffffffffff61004660201b60201c565b610291565b600073ffffffffffff" +
    "ffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffffffffff161415" +
    "6100e9576040517f08c379a000000000000000000000000000000000000000000000000000000000" +
    "815260040180806020018281038252601f8152602001807f45524332303a206d696e7420746f2074" +
    "6865207a65726f20616464726573730081525060200191505060405180910390fd5b610102816002" +
    "5461020960201b610c7c1790919060201c565b60028190555061015d816000808573ffffffffffff" +
    "ffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260" +
    "20019081526020016000205461020960201b610c7c1790919060201c565b6000808473ffffffffff" +
    "ffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152" +
    "602001908152602001600020819055508173ffffffffffffffffffffffffffffffffffffffff1660" +
    "0073ffffffffffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa" +
    "952ba7f163c4a11628f55a4df523b3ef836040518082815260200191505060405180910390a35050" +
    "565b600080828401905083811015610287576040517f08c379a00000000000000000000000000000" +
    "0000000000000000000000000000815260040180806020018281038252601b8152602001807f5361" +
    "66654d6174683a206164646974696f6e206f766572666c6f77000000000081525060200191505060" +
    "405180910390fd5b8091505092915050565b610e3a806102a06000396000f3fe6080604052348015" +
    "61001057600080fd5b50600436106100885760003560e01c806370a082311161005b57806370a082" +
    "31146101fd578063a457c2d714610255578063a9059cbb146102bb578063dd62ed3e146103215761" +
    "0088565b8063095ea7b31461008d57806318160ddd146100f357806323b872dd1461011157806339" +
    "50935114610197575b600080fd5b6100d9600480360360408110156100a357600080fd5b81019080" +
    "803573ffffffffffffffffffffffffffffffffffffffff1690602001909291908035906020019092" +
    "9190505050610399565b604051808215151515815260200191505060405180910390f35b6100fb61" +
    "03b7565b6040518082815260200191505060405180910390f35b61017d6004803603606081101561" +
    "012757600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff1690602001" +
    "90929190803573ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060" +
    "2001909291905050506103c1565b604051808215151515815260200191505060405180910390f35b" +
    "6101e3600480360360408110156101ad57600080fd5b81019080803573ffffffffffffffffffffff" +
    "ffffffffffffffffff1690602001909291908035906020019092919050505061049a565b60405180" +
    "8215151515815260200191505060405180910390f35b61023f600480360360208110156102135760" +
    "0080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff169060200190929190" +
    "50505061054d565b6040518082815260200191505060405180910390f35b6102a160048036036040" +
    "81101561026b57600080fd5b81019080803573ffffffffffffffffffffffffffffffffffffffff16" +
    "906020019092919080359060200190929190505050610595565b6040518082151515158152602001" +
    "91505060405180910390f35b610307600480360360408110156102d157600080fd5b810190808035" +
    "73ffffffffffffffffffffffffffffffffffffffff16906020019092919080359060200190929190" +
    "505050610662565b604051808215151515815260200191505060405180910390f35b610383600480" +
    "3603604081101561033757600080fd5b81019080803573ffffffffffffffffffffffffffffffffff" +
    "ffffff169060200190929190803573ffffffffffffffffffffffffffffffffffffffff1690602001" +
    "90929190505050610680565b6040518082815260200191505060405180910390f35b60006103ad61" +
    "03a6610707565b848461070f565b6001905092915050565b6000600254905090565b60006103ce84" +
    "8484610906565b61048f846103da610707565b61048a856040518060600160405280602881526020" +
    "01610d7060289139600160008b73ffffffffffffffffffffffffffffffffffffffff1673ffffffff" +
    "ffffffffffffffffffffffffffffffff168152602001908152602001600020600061044061070756" +
    "5b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffff" +
    "ffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b61070f565b" +
    "600190509392505050565b60006105436104a7610707565b8461053e85600160006104b861070756" +
    "5b73ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffff" +
    "ffffffff16815260200190815260200160002060008973ffffffffffffffffffffffffffffffffff" +
    "ffffff1673ffffffffffffffffffffffffffffffffffffffff168152602001908152602001600020" +
    "54610c7c90919063ffffffff16565b61070f565b6001905092915050565b60008060008373ffffff" +
    "ffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff16" +
    "8152602001908152602001600020549050919050565b60006106586105a2610707565b8461065385" +
    "604051806060016040528060258152602001610de160259139600160006105cc610707565b73ffff" +
    "ffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff" +
    "16815260200190815260200160002060008a73ffffffffffffffffffffffffffffffffffffffff16" +
    "73ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160002054610bbc" +
    "9092919063ffffffff16565b61070f565b6001905092915050565b600061067661066f610707565b" +
    "8484610906565b6001905092915050565b6000600160008473ffffffffffffffffffffffffffffff" +
    "ffffffffff1673ffffffffffffffffffffffffffffffffffffffff16815260200190815260200160" +
    "002060008373ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffff" +
    "ffffffffffffffff16815260200190815260200160002054905092915050565b600033905090565b" +
    "600073ffffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffff" +
    "ffffffffffff161415610795576040517f08c379a000000000000000000000000000000000000000" +
    "0000000000000000008152600401808060200182810382526024815260200180610dbd6024913960" +
    "400191505060405180910390fd5b600073ffffffffffffffffffffffffffffffffffffffff168273" +
    "ffffffffffffffffffffffffffffffffffffffff16141561081b576040517f08c379a00000000000" +
    "00000000000000000000000000000000000000000000008152600401808060200182810382526022" +
    "815260200180610d286022913960400191505060405180910390fd5b80600160008573ffffffffff" +
    "ffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff168152" +
    "60200190815260200160002060008473ffffffffffffffffffffffffffffffffffffffff1673ffff" +
    "ffffffffffffffffffffffffffffffffffff168152602001908152602001600020819055508173ff" +
    "ffffffffffffffffffffffffffffffffffffff168373ffffffffffffffffffffffffffffffffffff" +
    "ffff167f8c5be1e5ebec7d5bd14f71427d1e84f3dd0314c0f7b2291e5b200ac8c7c3b92583604051" +
    "8082815260200191505060405180910390a3505050565b600073ffffffffffffffffffffffffffff" +
    "ffffffffffff168373ffffffffffffffffffffffffffffffffffffffff16141561098c576040517f" +
    "08c379a0000000000000000000000000000000000000000000000000000000008152600401808060" +
    "200182810382526025815260200180610d986025913960400191505060405180910390fd5b600073" +
    "ffffffffffffffffffffffffffffffffffffffff168273ffffffffffffffffffffffffffffffffff" +
    "ffffff161415610a12576040517f08c379a000000000000000000000000000000000000000000000" +
    "0000000000008152600401808060200182810382526023815260200180610d056023913960400191" +
    "505060405180910390fd5b610a7d81604051806060016040528060268152602001610d4a60269139" +
    "6000808773ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffff" +
    "ffffffffffffff16815260200190815260200160002054610bbc9092919063ffffffff16565b6000" +
    "808573ffffffffffffffffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffff" +
    "ffffffffff16815260200190815260200160002081905550610b10816000808573ffffffffffffff" +
    "ffffffffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020" +
    "0190815260200160002054610c7c90919063ffffffff16565b6000808473ffffffffffffffffffff" +
    "ffffffffffffffffffff1673ffffffffffffffffffffffffffffffffffffffff1681526020019081" +
    "52602001600020819055508173ffffffffffffffffffffffffffffffffffffffff168373ffffffff" +
    "ffffffffffffffffffffffffffffffff167fddf252ad1be2c89b69c2b068fc378daa952ba7f163c4" +
    "a11628f55a4df523b3ef836040518082815260200191505060405180910390a3505050565b600083" +
    "8311158290610c69576040517f08c379a00000000000000000000000000000000000000000000000" +
    "00000000008152600401808060200182810382528381815181526020019150805190602001908083" +
    "8360005b83811015610c2e578082015181840152602081019050610c13565b505050509050908101" +
    "90601f168015610c5b5780820380516001836020036101000a031916815260200191505b50925050" +
    "5060405180910390fd5b5060008385039050809150509392505050565b6000808284019050838110" +
    "15610cfa576040517f08c379a0000000000000000000000000000000000000000000000000000000" +
    "00815260040180806020018281038252601b8152602001807f536166654d6174683a206164646974" +
    "696f6e206f766572666c6f77000000000081525060200191505060405180910390fd5b8091505092" +
    "91505056fe45524332303a207472616e7366657220746f20746865207a65726f2061646472657373" +
    "45524332303a20617070726f766520746f20746865207a65726f206164647265737345524332303a" +
    "207472616e7366657220616d6f756e7420657863656564732062616c616e636545524332303a2074" +
    "72616e7366657220616d6f756e74206578636565647320616c6c6f77616e636545524332303a2074" +
    "72616e736665722066726f6d20746865207a65726f206164647265737345524332303a2061707072" +
    "6f76652066726f6d20746865207a65726f206164647265737345524332303a206465637265617365" +
    "6420616c6c6f77616e63652062656c6f77207a65726fa265627a7a72315820c7a5ffabf642bda147" +
    "00b2de42f8c57b36621af020441df825de45fd2b3e1c5c64736f6c63430005100032";

  async function sendTransaction(context) {
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: TEST_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x" + EXPECTED_TRANSACTION_GAS_COST.toString(16),
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );

    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    return tx;
  }

  step("should create a Log filter and return the ID", async function () {
    let create_filter = await customRequest(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: [
          "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
          "0x5c4242beB94dE30b922f57241f1D02f36e906915",
        ],
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(create_filter.result).to.be.eq("0x1");
  });

  step("should increment filter ID", async function () {
    let create_filter = await customRequest(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x1",
        toBlock: "0x2",
        address: "0xC2Bf5F29a4384b1aB0C063e1c666f02121B6084a",
        topics: ["0xddf252ad1be2c89b69c2b068fc378daa952ba7f163c4a11628f55a4df523b3ef"],
      },
    ]);
    expect(create_filter.result).to.be.eq("0x2");
  });

  step("should create a Block filter and return the ID", async function () {
    let create_filter = await customRequest(context.web3, "eth_newBlockFilter", []);
    expect(create_filter.result).to.be.eq("0x3");
  });

  step(
    "should return unsupported error for Pending Transaction filter creation",
    async function () {
      let r = await customRequest(context.web3, "eth_newPendingTransactionFilter", []);
      expect(r.error).to.include({
        message: "Method not available.",
      });
    }
  );

  step("should return responses for Block filter polling.", async function () {
    let block = await context.web3.eth.getBlock(0);
    let poll = await customRequest(context.web3, "eth_getFilterChanges", ["0x3"]);

    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0]).to.be.eq(block.hash);

    await createAndFinalizeBlock(context.polkadotApi);

    block = await context.web3.eth.getBlock(1);
    poll = await customRequest(context.web3, "eth_getFilterChanges", ["0x3"]);

    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0]).to.be.eq(block.hash);

    await createAndFinalizeBlock(context.polkadotApi);
    await createAndFinalizeBlock(context.polkadotApi);

    block = await context.web3.eth.getBlock(2);
    let block_b = await context.web3.eth.getBlock(3);
    poll = await customRequest(context.web3, "eth_getFilterChanges", ["0x3"]);

    expect(poll.result.length).to.be.eq(2);
    expect(poll.result[0]).to.be.eq(block.hash);
    expect(poll.result[1]).to.be.eq(block_b.hash);
  });

  step("should return responses for Log filter polling.", async function () {
    // Create contract.
    let tx = await sendTransaction(context);
    await createAndFinalizeBlock(context.polkadotApi);
    let receipt = await context.web3.eth.getTransactionReceipt(tx.transactionHash);

    expect(receipt.logs.length).to.be.eq(1);

    // Create a filter for the created contract.
    let create_filter = await customRequest(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: receipt.contractAddress,
        topics: receipt.logs[0].topics,
      },
    ]);
    let poll = await customRequest(context.web3, "eth_getFilterChanges", [create_filter.result]);

    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0].address.toLowerCase()).to.be.eq(receipt.contractAddress.toLowerCase());
    expect(poll.result[0].topics).to.be.deep.eq(receipt.logs[0].topics);

    // A subsequent request must be empty.
    poll = await customRequest(context.web3, "eth_getFilterChanges", [create_filter.result]);
    expect(poll.result.length).to.be.eq(0);
  });

  step("should return response for raw Log filter request.", async function () {
    // Create contract.
    let tx = await sendTransaction(context);
    await createAndFinalizeBlock(context.polkadotApi);
    let receipt = await context.web3.eth.getTransactionReceipt(tx.transactionHash);

    expect(receipt.logs.length).to.be.eq(1);

    // Create a filter for the created contract.
    let create_filter = await customRequest(context.web3, "eth_newFilter", [
      {
        fromBlock: "0x0",
        toBlock: "latest",
        address: receipt.contractAddress,
        topics: receipt.logs[0].topics,
      },
    ]);
    let poll = await customRequest(context.web3, "eth_getFilterLogs", [create_filter.result]);

    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0].address.toLowerCase()).to.be.eq(receipt.contractAddress.toLowerCase());
    expect(poll.result[0].topics).to.be.deep.eq(receipt.logs[0].topics);

    // A subsequent request must return the same response.
    poll = await customRequest(context.web3, "eth_getFilterLogs", [create_filter.result]);

    expect(poll.result.length).to.be.eq(1);
    expect(poll.result[0].address.toLowerCase()).to.be.eq(receipt.contractAddress.toLowerCase());
    expect(poll.result[0].topics).to.be.deep.eq(receipt.logs[0].topics);
  });

  step("should uninstall created filters.", async function () {
    let create_filter = await customRequest(context.web3, "eth_newBlockFilter", []);
    let filter_id = create_filter.result;

    // Should return true when removed from the filter pool.
    let uninstall = await customRequest(context.web3, "eth_uninstallFilter", [filter_id]);
    expect(uninstall.result).to.be.eq(true);

    // Should return error if does not exist.
    let r = await customRequest(context.web3, "eth_uninstallFilter", [filter_id]);
    expect(r.error).to.include({
      message: "Filter id 6 does not exist.",
    });
  });

  step("should drain the filter pool.", async function () {
    this.timeout(15000);
    const block_lifespan_threshold = 100;

    let create_filter = await customRequest(context.web3, "eth_newBlockFilter", []);
    let filter_id = create_filter.result;

    for (let i = 0; i <= block_lifespan_threshold; i++) {
      await createAndFinalizeBlock(context.polkadotApi);
    }

    let r = await customRequest(context.web3, "eth_getFilterChanges", [filter_id]);
    expect(r.error).to.include({
      message: "Filter id 6 does not exist.",
    });
  });

  step("should have a filter pool max size of 500.", async function () {
    const max_filter_pool = 500;

    for (let i = 0; i < max_filter_pool; i++) {
      await customRequest(context.web3, "eth_newBlockFilter", []);
    }

    let r = await customRequest(context.web3, "eth_newBlockFilter", []);
    expect(r.error).to.include({
      message: "Filter pool is full (limit 500).",
    });
  });
});
