import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { ERC20_BYTECODE } from "./constants/testContracts";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

let txContract, contractAddress;

describeWithMoonbeam("Moonbeam RPC (TxPool RPC module)", `simple-specs.json`, (context) => {
  describe("TxPool test on contract creation", async () => {
    let txContract, nonce, tx;
    beforeEach(async () => {
      nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: ERC20_BYTECODE,
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      txContract = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    });
    it("should get pending pool information on Create", async function () {
      let inspect = await customRequest(context.web3, "txpool_inspect", []);
      let data = inspect.result.pending[GENESIS_ACCOUNT][context.web3.utils.toHex(nonce)];
      expect(data).to.not.be.undefined;
      expect(data).to.be.equal(
        "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 1 wei"
      );

      let content = await customRequest(context.web3, "txpool_content", []);

      data = content.result.pending[GENESIS_ACCOUNT][context.web3.utils.toHex(nonce)];
      expect(data).to.include({
        blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
        blockNumber: null,
        from: GENESIS_ACCOUNT.toString(),
        gas: "0x100000",
        gasPrice: "0x1",
        hash: tx.messageHash.toString(),
        nonce: context.web3.utils.toHex(nonce),
        to: "0x0000000000000000000000000000000000000000",
        value: "0x0",
      });
      await createAndFinalizeBlock(context.polkadotApi);
    });

    it("pool should be empty after producing a block", async function () {
      await createAndFinalizeBlock(context.polkadotApi);
      const receipt = await context.web3.eth.getTransactionReceipt(txContract.result);
      contractAddress = receipt.contractAddress;

      let inspect = await customRequest(context.web3, "txpool_inspect", []);
      let data = inspect.result.pending[GENESIS_ACCOUNT];
      expect(data).to.be.undefined;

      let content = await customRequest(context.web3, "txpool_content", []);
      data = content.result.pending[GENESIS_ACCOUNT];
      expect(data).to.be.undefined;
    });
  });

  describe("TxPool test on contract call and finalization", async () => {
    let txContract, contractAddress;
    before(async function () {
      const tx = await context.web3.eth.accounts.signTransaction(
        {
          from: GENESIS_ACCOUNT,
          data: ERC20_BYTECODE,
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      txContract = await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
      await createAndFinalizeBlock(context.polkadotApi);
      const receipt = await context.web3.eth.getTransactionReceipt(txContract.result);

      contractAddress = receipt.contractAddress;
    });
    it("should get pending pool information on Call", async function () {
      this.timeout(15000);

      const transferFnCode = `a9059cbb000000000000000000000000`;
      const tokensToTransfer = `00000000000000000000000000000000000000000000000000000000000000dd`;
      const inputCode = `0x${transferFnCode}${GENESIS_ACCOUNT.substring(2)}${tokensToTransfer}`;
      const nonce = await context.web3.eth.getTransactionCount(GENESIS_ACCOUNT);
      const tx = await context.web3.eth.accounts.signTransaction(
        {
          to: contractAddress,
          data: inputCode,
          value: "0x00",
          gasPrice: "0x01",
          gas: "0x100000",
        },
        GENESIS_ACCOUNT_PRIVATE_KEY
      );
      await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);

      let inspect = await customRequest(context.web3, "txpool_inspect", []);
      let data = inspect.result.pending[GENESIS_ACCOUNT][context.web3.utils.toHex(nonce)];

      expect(data).to.not.be.undefined;
      expect(data).to.be.equal(
        contractAddress.toString().toLowerCase() + ": 0 wei + 1048576 gas x 1 wei"
      );

      let content = await customRequest(context.web3, "txpool_content", []);
      data = content.result.pending[GENESIS_ACCOUNT][context.web3.utils.toHex(nonce)];
      expect(data).to.include({
        blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
        blockNumber: null,
        from: GENESIS_ACCOUNT.toString(),
        gas: "0x100000",
        gasPrice: "0x1",
        hash: tx.messageHash.toString(),
        nonce: context.web3.utils.toHex(nonce),
        to: contractAddress.toString().toLowerCase(),
        value: "0x0",
      });
    });
  });
});
