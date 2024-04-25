import { beforeEach, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  alith,
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  GLMR,
} from "@moonwall/util";
import { encodeFunctionData, parseEther } from "viem";
import {
  expectEVMResult,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  XcmFragment,
  XcmFragmentConfig,
} from "../../../../helpers";

describeSuite({
  id: "D010611",
  title: "ERC20 interactionss",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    let contract: `0x${string}`;

    beforeEach(async function () {
      const { contractAddress } = await context.deployContract!("ERC20Sample");
      contract = contractAddress;
    });

    it({
      id: "T01",
      title: "Should get the greeter message from the contract",
      test: async function () {
        const response = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "greeter",
        });

        expect(response).toEqual("Hello, ERC20!");
      },
    });

    it({
      id: "T02",
      title: "Should mint as expected",
      test: async function () {
        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "mint",
          args: [BALTATHAR_ADDRESS, 10n * GLMR],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const bal = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
        });

        expect(result?.successful).toBe(true);
        expect(bal).toEqual(10n * GLMR);
      },
    });

    it({
      id: "T03",
      title: "Should burn as expected",
      test: async function () {
        const amount = 10n * GLMR;

        const balBefore = (await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        })) as bigint;

        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "burn",
          args: [amount],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const balAfter = (await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [ALITH_ADDRESS],
        })) as bigint;

        expect(result?.successful).toBe(true);
        expect(balBefore - balAfter).toEqual(amount);
      },
    });

    it({
      id: "T04",
      title: "Should approve as expected",
      test: async function () {
        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "approve",
          args: [BALTATHAR_ADDRESS, 7n * GLMR],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const approval = (await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "allowance",
          args: [ALITH_ADDRESS, BALTATHAR_ADDRESS],
        })) as bigint;

        expect(result?.successful).toBe(true);
        expect(approval).toEqual(7n * GLMR);
      },
    });

    it({
      id: "T05",
      title: "Should transfer as expected",
      test: async function () {
        const tx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "transfer",
          args: [BALTATHAR_ADDRESS, 10n * GLMR],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(tx);

        const bal = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
        });

        expect(result?.successful).toBe(true);
        expect(bal).toEqual(10n * GLMR);
      },
    });

    it({
      id: "T06",
      title: "Should transferFrom as expected",
      test: async function () {
        await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "approve",
          args: [CHARLETH_ADDRESS, 3n * GLMR],
        });
        await context.createBlock();

        await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "transferFrom",
          args: [ALITH_ADDRESS, BALTATHAR_ADDRESS, 3n * GLMR],
          privateKey: CHARLETH_PRIVATE_KEY,
        });
        await context.createBlock();

        const bal = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "balanceOf",
          args: [BALTATHAR_ADDRESS],
        });

        expect(bal).toEqual(3n * GLMR);
      },
    });

    // TODO mint via XCM
    it({
      id: "T07",
      title: "Should mint as expected",
      test: async function () {
        const { abi } = fetchCompiledContract("ERC20Sample");
        const paraId = 888;
        const paraSovereign = sovereignAccountOfSibling(context, paraId);
        const amountTransferred = 1_000n;

        const metadata = await context.pjsApi.rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();
        const erc20XcmPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Erc20XcmBridge")!
          .index.toNumber();

        // Send some native tokens to the sovereign account of paraId (to pay fees)
        await context.pjsApi.tx.balances
          .transferAllowDeath(paraSovereign, parseEther("1"))
          .signAndSend(alith);
        await context.createBlock();

        // Send some erc20 tokens to the sovereign account of paraId
        const rawTx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "transfer",
          args: [paraSovereign, amountTransferred],
          rawTxOnly: true,
        });

        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        expect(
          await context.readContract!({
            contractName: "ERC20Sample",
            contractAddress: contract,
            functionName: "balanceOf",
            args: [paraSovereign],
          })
        ).equals(amountTransferred);

        // Generate call data
        const callData = encodeFunctionData({
          abi,
          functionName: "mint",
          args: [BALTATHAR_ADDRESS, 30n * GLMR],
        });

        const gasLimit = await context.viem().estimateGas({
          account: ALITH_ADDRESS,
          to: contract,
          data: callData,
        });

        const subTx = context.pjsApi.tx.ethereumXcm.transact({
          V2: {
            gasLimit: gasLimit + 10000n,
            action: { Call: contract },
            input: callData,
          },
        });

        const encodedCall = subTx.method.toHex();

        // Create the incoming xcm message
        const amountToWithdraw = BigInt(20 * 10 ** 16); // 0.01 DEV
        const weightTransact = 40000000000n; // 25000 * Gas limit of EVM call
        const devMultiLocation = { parents: 0, interior: { X1: { PalletInstance: 3 } } };
        // 3. XCM Instruction 1
        const instr1 = {
          WithdrawAsset: [
            {
              id: { Concrete: devMultiLocation },
              fun: { Fungible: amountToWithdraw },
            },
          ],
        };

        // 4. XCM Instruction 2
        const instr2 = {
          BuyExecution: {
            fees: {
              id: { Concrete: devMultiLocation },
              fun: { Fungible: amountToWithdraw },
            },
            weightLimit: { Unlimited: null },
          },
        };

        // 5. XCM Instruction 3
        const instr3 = {
          Transact: {
            // TODO CHANGE BELOW TO ALITH SIGNER
            originKind: "SovereignAccount",
            requireWeightAtMost: { refTime: weightTransact, proofSize: 700000n },
            call: {
              encoded: encodedCall,
            },
          },
        };

        // 6. XCM Instruction 4
        const instr4 = {
          DepositAsset: {
            assets: { Wild: "All" },
            beneficiary: {
              parents: 0,
              interior: { X1: { AccountKey20: { key: BALTATHAR_ADDRESS } } },
            },
          },
        };

        // 7. Build XCM Message
        const xcmMessage = { V3: [instr1, instr2, instr3, instr4] };

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage,
        });

        expect(
          await context.readContract!({
            contractName: "ERC20Sample",
            contractAddress: contract,
            functionName: "balanceOf",
            args: [BALTATHAR_ADDRESS],
          })
        ).equals(amountTransferred);
      },
    });

    // TODO burn via XCM

    // TODO transferFrom via XCM
  },
});
