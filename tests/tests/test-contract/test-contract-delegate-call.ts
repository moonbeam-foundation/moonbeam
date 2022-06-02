import "@moonbeam-network/api-augment";
import { expect } from "chai";
import { Contract } from "web3-eth-contract";
import { ethers } from "ethers";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract } from "../../util/transactions";
import { customWeb3Request } from "../../util/providers";
import { getCompiled } from "../../util/contracts";
import { alith } from "../../util/accounts";

describeDevMoonbeam("Delegate Call", (context) => {
  it("should work for normal smart contract", async function () {
    this.timeout(10000);

    const { contract: contractProxy, rawTx } = await createContract(context, "TestCallList");
    await context.createBlock({ transactions: [rawTx] });

    const { contract: contractDummy, rawTx: rawTx2 } = await createContract(
      context,
      "TestContract"
    );
    await context.createBlock({ transactions: [rawTx2] });

    const proxyInterface = new ethers.utils.Interface(
      (await getCompiled("TestCallList")).contract.abi
    );
    const dummyInterface = new ethers.utils.Interface(
      (await getCompiled("TestContract")).contract.abi
    );

    const tx_call = await customWeb3Request(context.web3, "eth_call", [
      {
        from: alith.address,
        to: contractProxy.options.address,
        gas: "0x100000",
        value: "0x00",
        data: proxyInterface.encodeFunctionData("delegateCall", [
          contractDummy.options.address,
          dummyInterface.encodeFunctionData("multiply", [42]),
        ]),
      },
    ]);

    console.log(JSON.stringify(tx_call));
    expect(tx_call.result).to.equal(
      "0x0000000000000000000000000000000000000000000000000000000000000001" +
        "0000000000000000000000000000000000000000000000000000000000000040" +
        "0000000000000000000000000000000000000000000000000000000000000020" +
        "0000000000000000000000000000000000000000000000000000000000000126"
    );
  });
});

describeDevMoonbeam("DELEGATECALL for precompiles", (context) => {
  let contractProxy: Contract;
  let proxyInterface: ethers.utils.Interface;

  const PRECOMPILE_PREFIXES = [
    1, 2, 3, 4, 5, 6, 7, 8, 9, 1024, 1025, 1026, 2048, 2049, 2050, 2051, 2052, 2053, 2054, 2055,
  ];

  // Ethereum precompile 1-9 are pure and allowed to be called through DELEGATECALL
  const ALLOWED_PRECOMPILE_PREFIXES = PRECOMPILE_PREFIXES.filter((add) => add <= 9);
  const FORBIDDEN_PRECOMPILE_PREFIXES = PRECOMPILE_PREFIXES.filter((add) => add > 9);
  const DELEGATECALL_FORDIDDEN_MESSAGE =
    "0x0000000000000000000000000000000000000000000000000000000000000000" +
    "0000000000000000000000000000000000000000000000000000000000000040" +
    "000000000000000000000000000000000000000000000000000000000000002e" +
    "63616e6e6f742062652063616c6c656420" + // cannot be called
    "776974682044454c454741544543414c4c20" + // with DELEGATECALL
    "6f722043414c4c434f4445" + // or CALLCODE
    "000000000000000000000000000000000000"; // padding

  before("Setup delecateCall contract", async () => {
    const contractDetails = await createContract(context, "TestCallList");
    contractProxy = contractDetails.contract;
    await context.createBlock({ transactions: [contractDetails.rawTx] });

    proxyInterface = new ethers.utils.Interface((await getCompiled("TestCallList")).contract.abi);
  });

  for (const precompilePrefix of ALLOWED_PRECOMPILE_PREFIXES) {
    it(`should succeed for standard precompile ${precompilePrefix}`, async function () {
      const precompileAddress = `0x${precompilePrefix.toString(16).padStart(40, "0")}`;
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
          to: contractProxy.options.address,
          gas: "0x200000",
          value: "0x00",
          data: proxyInterface.encodeFunctionData("delegateCall", [precompileAddress, "0x00"]),
        },
      ]);

      expect(tx_call.result).to.not.equal(DELEGATECALL_FORDIDDEN_MESSAGE);
    });
  }

  for (const precompilePrefix of FORBIDDEN_PRECOMPILE_PREFIXES) {
    it(`should fail for non-standard precompile ${precompilePrefix}`, async function () {
      const precompileAddress = `0x${precompilePrefix.toString(16).padStart(40, "0")}`;
      const tx_call = await customWeb3Request(context.web3, "eth_call", [
        {
          from: alith.address,
          to: contractProxy.options.address,
          gas: "0x100000",
          value: "0x00",
          data: proxyInterface.encodeFunctionData("delegateCall", [precompileAddress, "0x00"]),
        },
      ]);

      expect(tx_call.result).to.equal(DELEGATECALL_FORDIDDEN_MESSAGE);
    });
  }
});
