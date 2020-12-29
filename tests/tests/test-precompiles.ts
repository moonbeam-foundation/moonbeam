import { expect } from "chai";
import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

// All test for the RPC

describeWithMoonbeam("Moonbeam (Precompiles)", `simple-specs.json`, (context) => {
  /*
   * pragma solidity ^0.7.0;
   * contract HashRipmd160{
   *   constructor() {
   *     require(ripemd160(bytes ('Hello World!')) ==
   *       hex'8476ee4631b9b30ac2754b0ee0c47e161d3f724c');
   *   }
   * }
   */
  const RIPEMD160_CONTRACT_BYTECODE =
    "608060405234801561001057600080fd5b507f8476ee4631b9b30ac275" +
    "4b0ee0c47e161d3f724c00000000000000000000000060036040518060400160405280600c81526020017f48656c" +
    "6c6f20576f726c642100000000000000000000000000000000000000008152506040518082805190602001908083" +
    "835b6020831061009d578051825260208201915060208101905060208303925061007a565b600183602003610100" +
    "0a038019825116818451168082178552505050505050905001915050602060405180830381855afa1580156100df" +
    "573d6000803e3d6000fd5b5050506040515160601b6bffffffffffffffffffffffff19161461010257600080fd5b" +
    "603f806101106000396000f3fe6080604052600080fdfea26469706673582212202febccafbee65a134279d3397f" +
    "ecfc56a3d2125987802a91add0260c7efa94d264736f6c634300060c0033";

  /*
   * pragma solidity ^0.7.0;
   *
   * contract ModularCheck {
   *
   *     // Verify simple modular exponentiation
   *     constructor() {
   *         require(modExp(3, 5, 7) == 5);
   *         require(modExp(5, 7, 11) == 3);
   *     }
   *
   *     // Wrapper function to use the precompile.
   *     // Taken from https://ethereum.stackexchange.com/a/71590/9963
   *     function modExp(uint256 _b, uint256 _e, uint256 _m) public returns (uint256 result) {
   *         assembly {
   *             // Free memory pointer
   *             let pointer := mload(0x40)
   *
   *             // Define length of base, exponent and modulus. 0x20 == 32 bytes
   *             mstore(pointer, 0x20)
   *             mstore(add(pointer, 0x20), 0x20)
   *             mstore(add(pointer, 0x40), 0x20)
   *
   *             // Define variables base, exponent and modulus
   *             mstore(add(pointer, 0x60), _b)
   *             mstore(add(pointer, 0x80), _e)

   *             mstore(add(pointer, 0xa0), _m)
   *             // Store the result
   *             let value := mload(0xc0)
   *
   *             // Call the precompiled contract 0x05 = bigModExp
   *             if iszero(call(not(0), 0x05, 0, pointer, 0xc0, value, 0x20)) {
   *                 revert(0, 0)
   *             }
   *
   *             result := mload(value)
   *         }
   *     }
   * }
   */
  const MODEXP_CONTRACT_BYTECODE =
    "608060405234801561001057600080fd5b50600561002760036005600761005660201b60201c565b146100315760" +
    "0080fd5b600361004760056007600b61005660201b60201c565b1461005157600080fd5b6100a5565b6000604051" +
    "60208152602080820152602060408201528460608201528360808201528260a082015260c05160208160c0846000" +
    "6005600019f161009857600080fd5b8051925050509392505050565b610104806100b46000396000f3fe60806040" +
    "52348015600f57600080fd5b506004361060285760003560e01c80633148f14f14602d575b600080fd5b606a6004" +
    "8036036060811015604157600080fd5b810190808035906020019092919080359060200190929190803590602001" +
    "909291905050506080565b6040518082815260200191505060405180910390f35b60006040516020815260208082" +
    "0152602060408201528460608201528360808201528260a082015260c05160208160c08460006005600019f160c1" +
    "57600080fd5b805192505050939250505056fea26469706673582212204d7e7dcd400a3b0d5772d63f43f37a9855" +
    "d556cdcdf0f7991cb39169ce7871ce64736f6c63430007000033";
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY =
    "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

  it("ripemd160 should be valid", async function () {
    const tx_call = await customRequest(context.web3, "eth_call", [
      {
        from: GENESIS_ACCOUNT,
        value: "0x0",
        gas: "0x10000",
        gasPrice: "0x01",
        to: "0x0000000000000000000000000000000000000003",
        data: `0x${Buffer.from("Hello world!").toString("hex")}`,
      },
    ]);

    expect(tx_call.result).equals(
      "0x0000000000000000000000007f772647d88750add82d8e1a7a3e5c0902a346a3"
    );
  });

  it("ripemd160 is valid inside a contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: RIPEMD160_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    expect(await context.web3.eth.getCode("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a")).equals(
      "0x6080604052600080fdfea26469706673582212202febccafbee65a134279d3397fecfc56a3d21259" +
        "87802a91add0260c7efa94d264736f6c634300060c0033"
    );
  });

  it("ModExp is valid inside a contract", async function () {
    // See also the ModExp unit tests at
    // github.com/paritytech/frontier/blob/378221a4/frame/evm/precompile/modexp/src/lib.rs#L101
    this.timeout(15000);
    const { rawTransaction, transactionHash } = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: MODEXP_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);

    // The contract should deploy successfully and the receipt should show success.
    let receipt = await customRequest(context.web3, "eth_getTransactionReceipt", [transactionHash]);
    expect(receipt.result.status).equals("0x1");
  });
});
