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
  descendOriginFromAddress20,
  expectEVMResult,
  injectHrmpMessageAndSeal,
  sovereignAccountOfSibling,
  XcmFragment,
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

    it({
      id: "T07",
      title: "Should mint as expected",
      test: async function () {
        const paraId = 888;

        const { originAddress, descendOriginAddress } = descendOriginFromAddress20(
          context,
          ALITH_ADDRESS,
          paraId
        );
        const sendingAddress = originAddress;
        log(`Sending Address: ${sendingAddress}`);
        log(`Descend Origin Address: ${descendOriginAddress}`);

        // Get Pallet balances index
        const metadata = await context.polkadotJs().rpc.state.getMetadata();
        const balancesPalletIndex = metadata.asLatest.pallets
          .find(({ name }) => name.toString() == "Balances")!
          .index.toNumber();

        const { abi } = fetchCompiledContract("ERC20Sample");
        const mintAmount = 36n * GLMR;
        const originalOwner = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "owner",
        });
        log(`Contract Address: ${contract}`);
        log(`Original Owner: ${originalOwner}`);

        const changeOwnerTx = await context.writeContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "transferOwnership",
          args: [descendOriginAddress],
          rawTxOnly: true,
        });

        const { result: changeOwnerRes } = await context.createBlock(changeOwnerTx);
        expectEVMResult(changeOwnerRes!.events, "Succeed");

        const owner = await context.readContract!({
          contractName: "ERC20Sample",
          contractAddress: contract,
          functionName: "owner",
        });
        log(`Contract owner is now: ${owner}`);

        await context.createBlock(
          context.polkadotJs().tx.balances.transferAllowDeath(descendOriginAddress, GLMR),
          { allowFailures: false }
        );

        // The payload which will get executed by the EVM
        const callData = encodeFunctionData({
          abi,
          functionName: "mint",
          args: [BALTATHAR_ADDRESS, mintAmount],
        });

        const gasLimit = await context.viem().estimateGas({
          account: descendOriginAddress,
          to: contract,
          data: callData,
        });

        const subTx = context.pjsApi.tx.ethereumXcm.transact({
          V2: {
            gasLimit,
            action: { Call: contract },
            input: callData,
            access_list: null,
            value: 0n,
          },
        });

        const encodedCall = subTx.method.toHex();

        //  0.005 GLMR worth of fees (i've chosen this value arbitrarily)
        const amountToWithdraw = 3_000_000_000_000_000n;

        // ( EVM Call gas + overhead ) * gas-to-weight multiplier
        const weightTransact = (gasLimit + 5000n) * 25000n;

        const xcmMessage2 = new XcmFragment({
          assets: [
            {
              multilocation: {
                parents: 0,
                interior: {
                  X1: { PalletInstance: balancesPalletIndex },
                },
              },
              fungible: amountToWithdraw,
            },
          ],
          descend_origin: sendingAddress,
        })
          .descend_origin()
          .withdraw_asset()
          .buy_execution()
          .push_any({
            Transact: {
              originKind: "SovereignAccount",
              requireWeightAtMost: {
                refTime: weightTransact,
                proofSize: 700000n,
              },
              call: {
                encoded: encodedCall,
              },
            },
          })
          .as_v3();

        // Mock the reception of the xcm message
        await injectHrmpMessageAndSeal(context, paraId, {
          type: "XcmVersionedXcm",
          payload: xcmMessage2,
        });

        expect(
          await context.readContract!({
            contractName: "ERC20Sample",
            contractAddress: contract,
            functionName: "balanceOf",
            args: [BALTATHAR_ADDRESS],
          })
        ).equals(mintAmount);
      },
    });

    // TODO burn via XCM

    // TODO transferFrom via XCM
  },
});
