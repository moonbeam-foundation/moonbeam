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

  /* pragma solidity ^0.7.0;
   * contract Bn128Addition{
   *    constructor() {
   *        bool success;
   *        uint256[4] memory input = [
   *            0x2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee202839703,
   *            0x301d1d33be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c915,
   *            0x18b18acfb4c2c30276db5411368e7185b311dd124691610c5d3b74034e093dc9,
   *            0x063c909c4720840cb5134cb9f59fa749755796819658d32efc0d288198f37266
   *        ];
   *        uint256[2] memory result;
   *
   *        assembly {
   *            // 0x06     id of the bn256Add precompile
   *            // 0        number of ether to transfer
   *            // 128      size of call parameters, i.e. 128 bytes total
   *            // 64       size of return value, i.e. 64 bytes / 512 bit for a BN256 curve point
   *            success := call(not(0), 0x06, 0, input, 128, result, 64)
   *        }
   *        require(success, "elliptic curve addition failed");
   *        require(
   *            result[0] ==
   *            0x2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7ae5648e61d02268b1a0a9fb7,
   *        "failed");
   *        require(
   *            result[1] ==
   *            0x21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb204,
   *        "failed");
   *    }
   *}
   */

  const BN128ADD_CONTRACT_BYTECODE =
    "608060405234801561001057600080fd5b5060008060405180608001604" +
    "052807f2243525c5efd4b9c3d3c45ac0ca3fe4dd85e830a4ce6b65fa1eeaee20283970381526020017f301d1d3" +
    "3be6da8e509df21cc35964723180eed7532537db9ae5e7d48f195c91581526020017f18b18acfb4c2c30276db5" +
    "411368e7185b311dd124691610c5d3b74034e093dc981526020017f063c909c4720840cb5134cb9f59fa749755" +
    "796819658d32efc0d288198f3726681525090506100bf610296565b60408160808460006006600019f19250826" +
    "10142576040517f08c379a00000000000000000000000000000000000000000000000000000000081526004018" +
    "0806020018281038252601e8152602001807f656c6c6970746963206375727665206164646974696f6e2066616" +
    "96c6564000081525060200191505060405180910390fd5b7f2bd3e6d0f3b142924f5ca7b49ce5b9d54c4703d7a" +
    "e5648e61d02268b1a0a9fb78160006002811061017057fe5b6020020151146101e8576040517f08c379a000000" +
    "000000000000000000000000000000000000000000000000000815260040180806020018281038252600681526" +
    "02001807f6661696c6564000000000000000000000000000000000000000000000000000081525060200191505" +
    "060405180910390fd5b7f21611ce0a6af85915e2f1d70300909ce2e49dfad4a4619c8390cae66cefdb20481600" +
    "16002811061021657fe5b60200201511461028e576040517f08c379a0000000000000000000000000000000000" +
    "0000000000000000000000081526004018080602001828103825260068152602001807f6661696c65640000000" +
    "00000000000000000000000000000000000000000000081525060200191505060405180910390fd5b505050610" +
    "2b8565b6040518060400160405280600290602082028036833780820191505090505090565b603f806102c6600" +
    "0396000f3fe6080604052600080fdfea264697066735822122075fa7407f63bde9752715fbe31095ab6ad9273e" +
    "2d758ca548cdb9d581cc4fcd264736f6c63430007060033";

  /*pragma solidity ^0.7.0;
   * contract Bn128Addition{
   *     constructor() {
   *         bool success;
   *         uint256[3] memory input = [
   *             0x070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c,
   *             0x031b8ce914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc,
   *             0x30644e72e131a029b85045b68181585d97816a916871ca8d3c208c16d87cfd46
   *         ];
   *         uint256[2] memory result;
   *
   *         assembly {
   *             // 0x07     id of the bn256Mul precompile
   *             // 0        number of ether to transfer
   *             // 96      size of call parameters, i.e. 96 bytes total
   *             // 64      size of return value, i.e. 64 bytes / 512 bit for a BN256 curve point
   *             success := call(not(0), 0x07, 0, input, 96, result, 64)
   *         }
   *         require(success, "elliptic curve multiplication failed");
   *         require(
   *            result[0] ==
   *            0x025a6f4181d2b4ea8b724290ffb40156eb0adb514c688556eb79cdea0752c2bb,
   *         "failed");
   *         require(
   *            result[1] ==
   *            0x2eff3f31dea215f1eb86023a133a996eb6300b44da664d64251d05381bb8a02e,
   *         "failed");
   *     }
   * }
   */

  const BN128_MUL_CONTRACT_BYTECODE =
    "608060405234801561001057600080fd5b506000806040518060600160" +
    "4052807f070a8d6a982153cae4be29d434e8faef8a47b274a053f5a4ee2a6c9c13c31e5c81526020017f031b8c" +
    "e914eba3a9ffb989f9cdd5b0f01943074bf4f0f315690ec3cec6981afc81526020017f30644e72e131a029b850" +
    "45b68181585d97816a916871ca8d3c208c16d87cfd468152509050610099610253565b60408160608460006007" +
    "600019f19250826100ff576040517f08c379a00000000000000000000000000000000000000000000000000000" +
    "000081526004018080602001828103825260248152602001806102c26024913960400191505060405180910390" +
    "fd5b7f025a6f4181d2b4ea8b724290ffb40156eb0adb514c688556eb79cdea0752c2bb8160006002811061012d" +
    "57fe5b6020020151146101a5576040517f08c379a0000000000000000000000000000000000000000000000000" +
    "0000000081526004018080602001828103825260068152602001807f6661696c65640000000000000000000000" +
    "00000000000000000000000000000081525060200191505060405180910390fd5b7f2eff3f31dea215f1eb8602" +
    "3a133a996eb6300b44da664d64251d05381bb8a02e816001600281106101d357fe5b60200201511461024b5760" +
    "40517f08c379a00000000000000000000000000000000000000000000000000000000081526004018080602001" +
    "828103825260068152602001807f6661696c656400000000000000000000000000000000000000000000000000" +
    "0081525060200191505060405180910390fd5b505050610275565b604051806040016040528060029060208202" +
    "8036833780820191505090505090565b603f806102836000396000f3fe6080604052600080fdfea26469706673" +
    "5822122075cd0f518b5eecae53e271cd43201f7af10ba181ece35fe769e037d2ce152f9864736f6c6343000706" +
    "0033656c6c6970746963206375727665206d756c7469706c69636174696f6e206661696c6564";

  /*
   * pragma solidity ^0.7.0;
   * contract Bn128Pairing{
   *   constructor() {
   *    uint256[12] memory input = [
   *        0x2eca0c7238bf16e83e7a1e6c5d49540685ff51380f309842a98561558019fc02,
   *        0x03d3260361bb8451de5ff5ecd17f010ff22f5c31cdf184e9020b06fa5997db84,
   *        0x1213d2149b006137fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee,
   *        0x2276cf730cf493cd95d64677bbb75fc42db72513a4c1e387b476d056f80aa75f,
   *        0x21ee6226d31426322afcda621464d0611d226783262e21bb3bc86b537e986237,
   *        0x096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b6552ea5f,
   *        0x06967a1237ebfeca9aaae0d6d0bab8e28c198c5a339ef8a2407e31cdac516db9,
   *        0x22160fa257a5fd5b280642ff47b65eca77e626cb685c84fa6d3b6882a283ddd1,
   *        0x198e9393920d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c2,
   *        0x1800deef121f1e76426a00665e5c4479674322d4f75edadd46debd5cd992f6ed,
   *        0x090689d0585ff075ec9e99ad690c3395bc4b313370b38ef355acdadcd122975b,
   *        0x12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6cc0166fa7daa
   *     ];
   *     uint256[1] memory result;
   *     bool success;
   *     assembly {
   *         // 0x08     id of the bn256CheckPairing precompile
   *         // 0        number of ether to transfer
   *         // 0        since we have an array of fixed length, our input starts in 0
   *         // 384      size of call parameters, i.e. 12*256 bits == 384 bytes
   *         // 32       size of result (one 32 byte boolean!)
   *     success := call(sub(gas(), 2000), 0x08, 0, input, 384, result, 32)
   *     }
   *     require(success, "elliptic curve pairing failed");
   *     require(result[0] == 1, "failed");
   *   }
   * }
   */

  const BN128_PAIRING_CONTRACT_BYTECODE =
    "608060405234801561001057600080fd5b50600060405180610180" +
    "01604052807f2eca0c7238bf16e83e7a1e6c5d49540685ff51380f309842a98561558019fc0281526020017f03" +
    "d3260361bb8451de5ff5ecd17f010ff22f5c31cdf184e9020b06fa5997db8481526020017f1213d2149b006137" +
    "fcfb23036606f848d638d576a120ca981b5b1a5f9300b3ee81526020017f2276cf730cf493cd95d64677bbb75f" +
    "c42db72513a4c1e387b476d056f80aa75f81526020017f21ee6226d31426322afcda621464d0611d226783262e" +
    "21bb3bc86b537e98623781526020017f096df1f82dff337dd5972e32a8ad43e28a78a96a823ef1cd4debe12b65" +
    "52ea5f81526020017f06967a1237ebfeca9aaae0d6d0bab8e28c198c5a339ef8a2407e31cdac516db981526020" +
    "017f22160fa257a5fd5b280642ff47b65eca77e626cb685c84fa6d3b6882a283ddd181526020017f198e939392" +
    "0d483a7260bfb731fb5d25f1aa493335a9e71297e485b7aef312c281526020017f1800deef121f1e76426a0066" +
    "5e5c4479674322d4f75edadd46debd5cd992f6ed81526020017f090689d0585ff075ec9e99ad690c3395bc4b31" +
    "3370b38ef355acdadcd122975b81526020017f12c85ea5db8c6deb4aab71808dcb408fe3d1e7690c43d37b4ce6" +
    "cc0166fa7daa81525090506101ef610306565b600060208261018085600060086107d05a03f190508061027757" +
    "6040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020" +
    "018281038252601d8152602001807f656c6c69707469632063757276652070616972696e67206661696c656400" +
    "000081525060200191505060405180910390fd5b60018260006001811061028657fe5b6020020151146102fe57" +
    "6040517f08c379a000000000000000000000000000000000000000000000000000000000815260040180806020" +
    "01828103825260068152602001807f6661696c6564000000000000000000000000000000000000000000000000" +
    "000081525060200191505060405180910390fd5b505050610328565b6040518060200160405280600190602082" +
    "028036833780820191505090505090565b603f806103366000396000f3fe6080604052600080fdfea264697066" +
    "73582212202c2287769364a973256a3d99b3689d357bf3f403e981a27be30121349505bb5c64736f6c63430007" +
    "060033";
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

  it("bn128add is valid inside a contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: BN128ADD_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    var receipt = await context.web3.eth.getTransactionReceipt(tx.transactionHash);
    expect(await context.web3.eth.getCode(receipt.contractAddress)).equals(
      "0x6080604052600080fdfea264697066735822122075fa7407f63bde9752715fbe31095ab6ad9273e2" +
        "d758ca548cdb9d581cc4fcd264736f6c63430007060033"
    );
  });

  it("bn128mul is valid inside a contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: BN128_MUL_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    var receipt = await context.web3.eth.getTransactionReceipt(tx.transactionHash);
    expect(await context.web3.eth.getCode(receipt.contractAddress)).equals(
      "0x6080604052600080fdfea264697066735822122075cd0f518b5eecae53e271cd43201f7af10ba181" +
        "ece35fe769e037d2ce152f9864736f6c63430007060033"
    );
  });

  it("bn128Pairing is valid inside a contract", async function () {
    this.timeout(15000);
    const tx = await context.web3.eth.accounts.signTransaction(
      {
        from: GENESIS_ACCOUNT,
        data: BN128_PAIRING_CONTRACT_BYTECODE,
        value: "0x00",
        gasPrice: "0x01",
        gas: "0x100000",
      },
      GENESIS_ACCOUNT_PRIVATE_KEY
    );
    await customRequest(context.web3, "eth_sendRawTransaction", [tx.rawTransaction]);
    await createAndFinalizeBlock(context.polkadotApi);
    var receipt = await context.web3.eth.getTransactionReceipt(tx.transactionHash);
    expect(await context.web3.eth.getCode(receipt.contractAddress)).equals(
      "0x6080604052600080fdfea26469706673582212202c2287769364a973256a3d99b3689d357bf3f403e9" +
        "81a27be30121349505bb5c64736f6c63430007060033"
    );
  });
});
