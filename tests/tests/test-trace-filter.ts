import { expect } from "chai";

import { createAndFinalizeBlock, describeWithMoonbeam, customRequest } from "./util";

import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

const CONTRACT = require("./constants/TraceFilter.json");

describeWithMoonbeam("Moonbeam RPC (trace_filter)", `simple-specs.json`, (context) => {
  describe("Basic tracing tests", async () => {
    beforeEach(async function () {
      this.timeout(15000);
      let current_height = await context.web3.eth.getBlockNumber();
      // We need to create as many blocks as the current longest chain plus 1 to allow for previously inserted tx to enter
      if (current_height != 0) {
        for (var i = 0; i < current_height; i++) {
          await createAndFinalizeBlock(
            context.polkadotApi,
            await context.polkadotApi.rpc.chain.getBlockHash(i),
            false
          );
        }
      }
    });
    it("Replay succeeding CREATE", async function () {
      this.timeout(15000);
      // Deploy contract
      const contract = new context.web3.eth.Contract(CONTRACT.abi);
      const contract_deploy = contract.deploy({
        data: CONTRACT.bytecode,
        arguments: [false], // don't revert
      });

      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: contract_deploy.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x500000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      let current_height = await context.web3.eth.getBlockNumber();
      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );
      let contract_address = (await context.web3.eth.getTransactionReceipt(send.result))
        .contractAddress;

      // Perform RPC call.
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height + 1),
          toBlock: context.web3.utils.numberToHex(current_height + 1),
        },
      ]);

      expect(response.result.length).to.equal(1);
      expect(response.result[0].action.createMethod).to.equal("create");
      expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
      expect(response.result[0].action.gas).to.equal("0x4ffead");
      expect(response.result[0].action.input).to.be.a("string");
      expect(response.result[0].action.value).to.equal("0x0");
      expect(response.result[0].blockHash).to.be.a("string");
      expect(response.result[0].blockNumber).to.equal(current_height + 1);
      expect(response.result[0].result.address).to.equal(contract_address.toLowerCase());
      expect(response.result[0].result.code).to.be.a("string");
      expect(response.result[0].result.gasUsed).to.equal("0x153");
      expect(response.result[0].error).to.equal(undefined);
      expect(response.result[0].subtraces).to.equal(0);
      expect(response.result[0].traceAddress.length).to.equal(0);
      expect(response.result[0].transactionHash).to.equal(send.result);
      expect(response.result[0].transactionPosition).to.equal(0);
      expect(response.result[0].type).to.equal("create");
    });

    it("Replay reverting CREATE", async function () {
      // Deploy contract
      const contract = new context.web3.eth.Contract(CONTRACT.abi);
      const contract_deploy = contract.deploy({
        data: CONTRACT.bytecode,
        arguments: [true], // revert
      });

      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: contract_deploy.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x500000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

      let current_height = await context.web3.eth.getBlockNumber();

      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );
      // Perform RPC call.
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height + 1),
          toBlock: context.web3.utils.numberToHex(current_height + 1),
        },
      ]);

      expect(response.result.length).to.equal(1);
      expect(response.result[0].action.createMethod).to.equal("create");
      expect(response.result[0].action.from).to.equal("0x6be02d1d3665660d22ff9624b7be0551ee1ac91b");
      expect(response.result[0].action.gas).to.equal("0x4fff44");
      expect(response.result[0].action.input).to.be.a("string");
      expect(response.result[0].action.value).to.equal("0x0");
      expect(response.result[0].blockHash).to.be.a("string");
      expect(response.result[0].blockNumber).to.equal(current_height + 1);
      expect(response.result[0].result).to.equal(undefined);
      expect(response.result[0].error).to.equal("Reverted");
      expect(response.result[0].subtraces).to.equal(0);
      expect(response.result[0].traceAddress.length).to.equal(0);
      expect(response.result[0].transactionHash).to.equal(send.result);
      expect(response.result[0].transactionPosition).to.equal(0);
      expect(response.result[0].type).to.equal("create");
    });

    it("Multiple transactions in the same block + trace over multiple blocks", async function () {
      const contract = new context.web3.eth.Contract(CONTRACT.abi);

      // Deploy 2 more contracts
      for (var i = 0; i < 2; i++) {
        const contract_deploy = contract.deploy({
          data: CONTRACT.bytecode,
          arguments: [false], // don't revert
        });
        let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);

        const tx = await context.web3.eth.accounts.signTransaction(
          {
            nonce: nonce + i,
            from: GENESIS_ACCOUNT,
            data: contract_deploy.encodeABI(),
            value: "0x00",
            gasPrice: "0x01",
            gas: "0x100000",
          },
          GENESIS_ACCOUNT_PRIVATE_KEY
        );

        let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      }

      let current_height = await context.web3.eth.getBlockNumber();

      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );
      const contract_deploy = contract.deploy({
        data: CONTRACT.bytecode,
        arguments: [false], // don't revert
      });

      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: contract_deploy.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height + 1),
        false
      );

      // Perform RPC call.
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height + 1),
          toBlock: context.web3.utils.numberToHex(current_height + 2),
        },
      ]);

      expect(response.result.length).to.equal(3);
      expect(response.result[0].blockNumber).to.equal(current_height + 1);
      expect(response.result[0].transactionPosition).to.equal(0);
      expect(response.result[1].blockNumber).to.equal(current_height + 1);
      expect(response.result[1].transactionPosition).to.equal(1);
      expect(response.result[2].blockNumber).to.equal(current_height + 2);
      expect(response.result[2].transactionPosition).to.equal(0);
    });
  });

  describe("Test tracing with contract subcalls (we need three contracts deployed)", async () => {
    let address0, address1, address2;
    before(async function () {
      this.timeout(15000);
      let tx_hashes = [];
      // Create a forked chain
      let current_height = await context.web3.eth.getBlockNumber();
      // We need to create as many blocks as the current longest chain plus 1 to allow for previously inserted tx to enter
      if (current_height != 0) {
        for (var i = 0; i < current_height; i++) {
          await createAndFinalizeBlock(
            context.polkadotApi,
            await context.polkadotApi.rpc.chain.getBlockHash(i),
            false
          );
        }
      }
      // Deploy first contract
      const contract = new context.web3.eth.Contract(CONTRACT.abi);
      const contract_deploy = contract.deploy({
        data: CONTRACT.bytecode,
        arguments: [false], // don't revert
      });
      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: contract_deploy.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x500000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      tx_hashes.push(
        await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
      );
      current_height = await context.web3.eth.getBlockNumber();
      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );

      let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      // Deploy 2 more contracts
      for (var i = 0; i < 2; i++) {
        const contract_deploy = contract.deploy({
          data: CONTRACT.bytecode,
          arguments: [false], // don't revert
        });

        const tx = await context.web3.eth.accounts.signTransaction(
          {
            nonce: nonce + i,
            from: GENESIS_ACCOUNT,
            data: contract_deploy.encodeABI(),
            value: "0x00",
            gasPrice: "0x01",
            gas: "0x100000",
          },
          GENESIS_ACCOUNT_PRIVATE_KEY
        );

        tx_hashes.push(
          await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
        );
      }
      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height + 1),
        false
      );

      //Map contract addresses
      address0 = (await context.web3.eth.getTransactionReceipt(tx_hashes[0].result))
        .contractAddress;
      address1 = (await context.web3.eth.getTransactionReceipt(tx_hashes[1].result))
        .contractAddress;
      address2 = (await context.web3.eth.getTransactionReceipt(tx_hashes[2].result))
        .contractAddress;
    });
    it("Call with subcalls, some reverting", async function () {
      const contract = new context.web3.eth.Contract(CONTRACT.abi);

      const contract_call = contract.methods.subcalls(address1, address2);

      const tx = await context.web3.eth.accounts.signTransaction(
        {
          to: address0,
          from: GENESIS_ACCOUNT,
          data: contract_call.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x500000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      let send = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

      let current_height = await context.web3.eth.getBlockNumber();

      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );
      // Perform RPC call.
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height + 1),
          toBlock: context.web3.utils.numberToHex(current_height + 1),
        },
      ]);

      expect(response.result.length).to.equal(7);
      expect(response.result[0].subtraces).to.equal(2);
      expect(response.result[0].traceAddress).to.deep.equal([]);
      expect(response.result[1].subtraces).to.equal(2);
      expect(response.result[1].traceAddress).to.deep.equal([0]);
      expect(response.result[2].subtraces).to.equal(0);
      expect(response.result[2].traceAddress).to.deep.equal([0, 0]);
      expect(response.result[3].subtraces).to.equal(0);
      expect(response.result[3].traceAddress).to.deep.equal([0, 1]);
      expect(response.result[4].subtraces).to.equal(2);
      expect(response.result[4].traceAddress).to.deep.equal([1]);
      expect(response.result[5].subtraces).to.equal(0);
      expect(response.result[5].traceAddress).to.deep.equal([1, 0]);
      expect(response.result[6].subtraces).to.equal(0);
      expect(response.result[6].traceAddress).to.deep.equal([1, 1]);
    });
  });

  describe("Tests tracing with all previous contracts deployed and transactions executed", async () => {
    let address0, address1, address2;
    before(async function () {
      this.timeout(15000);
      let tx_hashes = [];

      let current_height = await context.web3.eth.getBlockNumber();
      // We need to create as many blocks as the current longest chain plus 1 to allow for previously inserted tx to enter

      if (current_height != 0) {
        for (var i = 0; i < current_height; i++) {
          await createAndFinalizeBlock(
            context.polkadotApi,
            await context.polkadotApi.rpc.chain.getBlockHash(i),
            false
          );
        }
      }

      // Deploy first contract
      let contract = new context.web3.eth.Contract(CONTRACT.abi);
      const contract_deploy = contract.deploy({
        data: CONTRACT.bytecode,
        arguments: [false], // don't revert
      });
      let tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: contract_deploy.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x500000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      tx_hashes.push(
        await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
      );
      current_height = await context.web3.eth.getBlockNumber();
      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );

      contract = new context.web3.eth.Contract(CONTRACT.abi);
      let nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      // Deploy 2 more contracts
      for (var i = 0; i < 2; i++) {
        const contract_deploy = contract.deploy({
          data: CONTRACT.bytecode,
          arguments: [false], // don't revert
        });

        tx = await context.web3.eth.accounts.signTransaction(
          {
            nonce: nonce + i,
            from: GENESIS_ACCOUNT,
            data: contract_deploy.encodeABI(),
            value: "0x00",
            gasPrice: "0x01",
            gas: "0x100000",
          },
          GENESIS_ACCOUNT_PRIVATE_KEY
        );

        tx_hashes.push(
          await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction])
        );
      }
      current_height = await context.web3.eth.getBlockNumber();

      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height),
        false
      );
      //Map contract addresses
      address0 = (await context.web3.eth.getTransactionReceipt(tx_hashes[0].result))
        .contractAddress;
      address1 = (await context.web3.eth.getTransactionReceipt(tx_hashes[1].result))
        .contractAddress;
      address2 = (await context.web3.eth.getTransactionReceipt(tx_hashes[2].result))
        .contractAddress;
      //transaction with subcalls
      contract = new context.web3.eth.Contract(CONTRACT.abi);

      const contract_call = contract.methods.subcalls(address1, address2);

      tx = await context.web3.eth.accounts.signTransaction(
        {
          to: address0,
          from: GENESIS_ACCOUNT,
          data: contract_call.encodeABI(),
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x500000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );

      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      await createAndFinalizeBlock(
        context.polkadotApi,
        await context.polkadotApi.rpc.chain.getBlockHash(current_height + 1),
        false
      );
    });
    it("Request range of blocks", async function () {
      let current_height = await context.web3.eth.getBlockNumber();

      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height - 1),
          toBlock: context.web3.utils.numberToHex(current_height),
        },
      ]);

      expect(response.result.length).to.equal(9);
      expect(response.result[0].blockNumber).to.equal(current_height - 1);
      expect(response.result[0].transactionPosition).to.equal(0);
      expect(response.result[1].blockNumber).to.equal(current_height - 1);
      expect(response.result[1].transactionPosition).to.equal(1);
      expect(response.result[2].blockNumber).to.equal(current_height);
      expect(response.result[2].transactionPosition).to.equal(0);
      expect(response.result[3].blockNumber).to.equal(current_height);
      expect(response.result[3].transactionPosition).to.equal(0);
      expect(response.result[4].blockNumber).to.equal(current_height);
      expect(response.result[4].transactionPosition).to.equal(0);
      expect(response.result[5].blockNumber).to.equal(current_height);
      expect(response.result[5].transactionPosition).to.equal(0);
      expect(response.result[6].blockNumber).to.equal(current_height);
      expect(response.result[6].transactionPosition).to.equal(0);
      expect(response.result[7].blockNumber).to.equal(current_height);
      expect(response.result[7].transactionPosition).to.equal(0);
      expect(response.result[8].blockNumber).to.equal(current_height);
      expect(response.result[8].transactionPosition).to.equal(0);
    });

    it("Filter fromAddress", async function () {
      let current_height = await context.web3.eth.getBlockNumber();
      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height - 1),
          toBlock: context.web3.utils.numberToHex(current_height),
          fromAddress: [GENESIS_ACCOUNT],
        },
      ]);

      expect(response.result.length).to.equal(3);
    });

    it("Filter toAddress", async function () {
      let current_height = await context.web3.eth.getBlockNumber();

      let response = await customRequest(context.web3, "trace_filter", [
        {
          fromBlock: context.web3.utils.numberToHex(current_height - 1),
          toBlock: context.web3.utils.numberToHex(current_height),
          toAddress: [address2],
        },
      ]);

      expect(response.result.length).to.equal(4);
    });
  });
});
