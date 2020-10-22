import { expect } from "chai";
import { createAndFinalizeBlock, customRequest, describeWithMoonbeam } from "./util";

// All test for the RPC

describeWithMoonbeam("Moonbeam (Precompiles)", `simple-specs.json`, (context) => {
  /*
  * pragma solidity ^0.7.0;
  * contract HashRipmd160{
  *   constructor() {
  *     require(ripemd160(bytes ('Hello World!')) == hex'8476ee4631b9b30ac2754b0ee0c47e161d3f724c');
  *   }
  * }
  */
  const RIPEMD160_CONTRACT_BYTECODE = "608060405234801561001057600080fd5b507f8476ee4631b9b30ac2754b0ee0c47e161d3f724c00000000000000000000000060036040518060400160405280600c81526020017f48656c6c6f20576f726c642100000000000000000000000000000000000000008152506040518082805190602001908083835b6020831061009d578051825260208201915060208101905060208303925061007a565b6001836020036101000a038019825116818451168082178552505050505050905001915050602060405180830381855afa1580156100df573d6000803e3d6000fd5b5050506040515160601b6bffffffffffffffffffffffff19161461010257600080fd5b603f806101106000396000f3fe6080604052600080fdfea26469706673582212202febccafbee65a134279d3397fecfc56a3d2125987802a91add0260c7efa94d264736f6c634300060c0033";
  const GENESIS_ACCOUNT = "0x6be02d1d3665660d22ff9624b7be0551ee1ac91b";
  const GENESIS_ACCOUNT_PRIVATE_KEY = "0x99B3C12287537E38C90A9219D4CB074A89A16E9CDB20BF85728EBD97C343E342";

    it("ripemd160 should be valid", async function () {
        const tx_call = await customRequest(
      context.web3,
      "eth_call",
      [{
        from: GENESIS_ACCOUNT,
        'value': "0x0",
        'gas': "0x10000",
        'gasPrice': "0x01",
        'to': '0x0000000000000000000000000000000000000003',
        'data': `0x${Buffer.from('Hello world!').toString('hex')}`
      }]);

    expect(tx_call.result).equals("0x0000000000000000000000007f772647d88750add82d8e1a7a3e5c0902a346a3");
    });



  // TODO: Restore this test once manual sealing is fixed https://purestake.atlassian.net/browse/MOON-81
    it.skip("ripemd160 is valid inside a contract", async function () {
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
    await createAndFinalizeBlock(context.web3);
    expect(await context.web3.eth.getCode("0xc2bf5f29a4384b1ab0c063e1c666f02121b6084a"))
      .equals("0x6080604052600080fdfea26469706673582212202febccafbee65a134279d3397fecfc56a3d2125987802a91add0260c7efa94d264736f6c634300060c0033");
    });
});
