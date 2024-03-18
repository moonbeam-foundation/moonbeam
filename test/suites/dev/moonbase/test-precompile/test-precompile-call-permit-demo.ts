import "@moonbeam-network/api-augment";
import {
  expect,
  beforeAll,
  describeSuite,
  fetchCompiledContract,
  deployCreateCompiledContract,
} from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  PRECOMPILE_CALL_PERMIT_ADDRESS,
  createViemTransaction,
} from "@moonwall/util";
import { Abi, encodeFunctionData, fromHex } from "viem";
import { expectEVMResult, getSignatureParameters } from "../../../../helpers";

describeSuite({
  id: "D012928",
  title: "Precompile - Call Permit - foo",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let callPermitDemoAbi: Abi;
    let callPermitDemoAddr: `0x${string}`;
    let callPermitAbi: Abi;

    beforeAll(async function () {
      const { abi: demoAbi, contractAddress } = await deployCreateCompiledContract(
        context,
        "CallPermitDemo",
        {
          gas: 5_000_000n,
        }
      );

      callPermitDemoAbi = demoAbi;
      callPermitDemoAddr = contractAddress;

      const { abi: precompileAbi } = fetchCompiledContract("CallPermit");
      callPermitAbi = precompileAbi;

      const bondAmount = (
        await context.viem().call({
          to: callPermitDemoAddr,
          data: encodeFunctionData({
            abi: callPermitDemoAbi,
            functionName: "BOND_AMOUNT",
          }),
        })
      ).data;

      const { result: baltatharResult } = await context.createBlock(
        createViemTransaction(context, {
          privateKey: BALTATHAR_PRIVATE_KEY,
          to: callPermitDemoAddr,
          data: encodeFunctionData({ abi: callPermitDemoAbi, functionName: "bond" }),
          value: fromHex(bondAmount!, "bigint"),
        })
      );
      expectEVMResult(baltatharResult!.events, "Succeed");

      // bond alice via baltathar using call permit
      const alithNonceResult = (
        await context.viem().call({
          to: PRECOMPILE_CALL_PERMIT_ADDRESS,
          data: encodeFunctionData({
            abi: callPermitAbi,
            functionName: "nonces",
            args: [ALITH_ADDRESS],
          }),
        })
      ).data;

      const signature = await context.viem().signTypedData({
        types: {
          EIP712Domain: [
            {
              name: "name",
              type: "string",
            },
            {
              name: "version",
              type: "string",
            },
            {
              name: "chainId",
              type: "uint256",
            },
            {
              name: "verifyingContract",
              type: "address",
            },
          ],
          CallPermit: [
            {
              name: "from",
              type: "address",
            },
            {
              name: "to",
              type: "address",
            },
            {
              name: "value",
              type: "uint256",
            },
            {
              name: "data",
              type: "bytes",
            },
            {
              name: "gaslimit",
              type: "uint64",
            },
            {
              name: "nonce",
              type: "uint256",
            },
            {
              name: "deadline",
              type: "uint256",
            },
          ],
        },
        primaryType: "CallPermit",
        domain: {
          name: "Call Permit Precompile",
          version: "1",
          chainId: 1281n,
          verifyingContract: PRECOMPILE_CALL_PERMIT_ADDRESS,
        },
        message: {
          from: ALITH_ADDRESS,
          to: callPermitDemoAddr,
          value: fromHex(bondAmount!, "bigint"),
          data: "0x",
          gaslimit: 100_000n,
          nonce: fromHex(alithNonceResult!, "bigint"),
          deadline: 9999999999n,
        },
      });
      const { v, r, s } = getSignatureParameters(signature);

      const { result: baltatharForAlithResult } = await context.createBlock(
        createViemTransaction(context, {
          privateKey: BALTATHAR_PRIVATE_KEY,
          to: callPermitDemoAddr,
          data: encodeFunctionData({
            abi: callPermitDemoAbi,
            functionName: "bondFor",
            args: [ALITH_ADDRESS, 100_000, 9999999999, v, r, s],
          }),
        })
      );
      expectEVMResult(baltatharForAlithResult!.events, "Succeed");
    });

    it({
      id: "T01",
      title: "should have bonds for baltathar and alith in contract balance",
      test: async function () {
        const freeBalance = (
          await context.polkadotJs().query.system.account(callPermitDemoAddr)
        ).data.free.toNumber();
        expect(freeBalance).to.equal(200);
      },
    });

    it({
      id: "T02",
      title: "should have bond for baltathar in contract storage",
      test: async function () {
        const baltatharBond = (
          await context.viem().call({
            to: callPermitDemoAddr,
            data: encodeFunctionData({
              abi: callPermitDemoAbi,
              functionName: "getBondAmount",
              args: [BALTATHAR_ADDRESS],
            }),
          })
        ).data;
        expect(fromHex(baltatharBond!, "bigint")).to.equal(100n);
      },
    });

    it({
      id: "T03",
      title: "should have bond for alith in contract storage",
      test: async function () {
        const alithBond = (
          await context.viem().call({
            to: callPermitDemoAddr,
            data: encodeFunctionData({
              abi: callPermitDemoAbi,
              functionName: "getBondAmount",
              args: [ALITH_ADDRESS],
            }),
          })
        ).data;
        expect(fromHex(alithBond!, "bigint")).to.equal(100n);
      },
    });
  },
});
