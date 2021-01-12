import { expect } from "chai";

import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";
import { ERC20_BYTECODE } from "./constants/testContracts";
import { GENESIS_ACCOUNT, GENESIS_ACCOUNT_PRIVATE_KEY } from "./constants";

let txContract, contractAddress;

describeWithMoonbeam("Moonbeam RPC (TxPool RPC module)", `simple-specs.json`, (context) => {
  it("should get pending pool information on Create", async function () {
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

    let inspect = await customRequest(context.web3, "txpool_inspect", []);

    let data = inspect.result.pending[GENESIS_ACCOUNT]["0x0"];
    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0x0000000000000000000000000000000000000000: 0 wei + 1048576 gas x 1 wei"
    );

    let content = await customRequest(context.web3, "txpool_content", []);

    data = content.result.pending[GENESIS_ACCOUNT]["0x0"];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gas: "0x100000",
      gasPrice: "0x1",
      hash: "0x6073b838b5cb04e04e57d6f25dd9387ff2a3d1de5c9a7bd89206f269cb64fe1b",
      nonce: "0x0",
      to: "0x0000000000000000000000000000000000000000",
      value: "0x0",
    });
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

  it("should get pending pool information on Call", async function () {
    const transferFnCode = `a9059cbb000000000000000000000000`;
    const tokensToTransfer = `00000000000000000000000000000000000000000000000000000000000000dd`;
    const inputCode = `0x${transferFnCode}${GENESIS_ACCOUNT.substring(2)}${tokensToTransfer}`;

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

    let data = inspect.result.pending[GENESIS_ACCOUNT]["0x1"];

    expect(data).to.not.be.undefined;
    expect(data).to.be.equal(
      "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a: 0 wei + 1048576 gas x 1 wei"
    );

    let content = await customRequest(context.web3, "txpool_content", []);

    data = content.result.pending[GENESIS_ACCOUNT]["0x1"];
    expect(data).to.include({
      blockHash: "0x0000000000000000000000000000000000000000000000000000000000000000",
      blockNumber: null,
      from: "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b",
      gas: "0x100000",
      gasPrice: "0x1",
      hash: "0x82e9940df25dec4f57030478ff848728e38e55cffd031cd6f43b3c114863544d",
      nonce: "0x1",
      to: "0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a",
      value: "0x0",
    });
  });
});
