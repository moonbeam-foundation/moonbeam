import "@moonbeam-network/api-augment";
import { describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  CONTRACT_PROXY_TYPE_ANY,
  CONTRACT_PROXY_TYPE_STAKING,
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  GLMR,
  GOLIATH_ADDRESS,
  GOLIATH_PRIVATE_KEY,
  PRECOMPILE_NATIVE_ERC20_ADDRESS,
  PRECOMPILE_PROXY_ADDRESS,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { encodeFunctionData, parseEther } from "viem";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { expectEVMResult } from "../../../../helpers";

describeSuite({
  id: "D012868",
  title: "Precompile - Proxy",
  foundationMethods: "dev",
  testCases: ({ it, log, context }) => {
    it({
      id: "T01",
      title: "should fail re-adding proxy account",
      test: async () => {
        await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [GOLIATH_ADDRESS, CONTRACT_PROXY_TYPE_STAKING, 0],
          privateKey: FAITH_PRIVATE_KEY,
        });
        await context.createBlock();

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [GOLIATH_ADDRESS, CONTRACT_PROXY_TYPE_STAKING, 0],
          privateKey: FAITH_PRIVATE_KEY,
          rawTxOnly: true,
          gas: 1_000_000n,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");
        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "Proxy",
              functionName: "addProxy",
              args: [GOLIATH_ADDRESS, CONTRACT_PROXY_TYPE_STAKING, 0],
              privateKey: FAITH_PRIVATE_KEY,
            })
        ).rejects.toThrowError("Cannot add more than one proxy");
      },
    });

    it({
      id: "T02",
      title: "should succeed with valid account",
      test: async () => {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [FAITH_ADDRESS, CONTRACT_PROXY_TYPE_STAKING, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const proxyAddedEvents = result!.events.reduce((acc, e) => {
          if (context.polkadotJs().events.proxy.ProxyAdded.is(e.event)) {
            acc.push({
              account: e.event.data[0].toString(),
              proxyType: e.event.data[2].toHuman(),
            });
          }
          return acc;
        }, []);

        expect(proxyAddedEvents).to.deep.equal([
          {
            account: ALITH_ADDRESS,
            proxyType: "Staking",
          },
        ]);
      },
    });

    it({
      id: "T03",
      title: "should fail if no existing proxy",
      test: async () => {
        const randomAddress = privateKeyToAccount(generatePrivateKey()).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "removeProxy",
          args: [randomAddress, CONTRACT_PROXY_TYPE_STAKING, 0],
          rawTxOnly: true,
          gas: 1_000_000n,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");

        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "Proxy",
              functionName: "removeProxy",
              args: [randomAddress, CONTRACT_PROXY_TYPE_STAKING, 0],
            })
        ).rejects.toThrowError('Some("NotFound")');
      },
    });

    it({
      id: "T04",
      title: "should succeed removing proxy if it exists",
      test: async () => {
        const randomAddress = privateKeyToAccount(generatePrivateKey()).address;

        await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [randomAddress, CONTRACT_PROXY_TYPE_STAKING, 0],
        });
        await context.createBlock();

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "removeProxy",
          args: [randomAddress, CONTRACT_PROXY_TYPE_STAKING, 0],
          rawTxOnly: true,
        });

        const expectEvents = [context.polkadotJs().events.proxy.ProxyRemoved];
        const { result } = await context.createBlock(rawTxn, { expectEvents, signer: alith });
        expectEVMResult(result!.events, "Succeed");
        const proxyRemovedEvents = result!.events.reduce((acc, e) => {
          if (context.polkadotJs().events.proxy.ProxyRemoved.is(e.event)) {
            acc.push({
              account: e.event.data[0].toString(),
              proxyType: e.event.data[2].toHuman(),
            });
          }
          return acc;
        }, []);

        expect(proxyRemovedEvents).to.deep.equal([
          {
            account: ALITH_ADDRESS,
            proxyType: "Staking",
          },
        ]);
      },
    });

    it({
      id: "T05",
      title: "should succeed removing all proxies even if none exist",
      test: async () => {
        const privateKey = generatePrivateKey();
        const randomAccount = privateKeyToAccount(privateKey);
        await context.createBlock(
          context.polkadotJs().tx.balances.transferAllowDeath(randomAccount.address, GLMR)
        );

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "removeProxies",
          privateKey,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");
      },
    });

    it({
      id: "T06",
      title: "should succeed removing all proxies",
      test: async () => {
        await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_STAKING, 0],
        });
        await context.createBlock();
        await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [CHARLETH_ADDRESS, CONTRACT_PROXY_TYPE_STAKING, 0],
        });
        await context.createBlock();

        const proxiesBefore = await context.polkadotJs().query.proxy.proxies(ALITH_ADDRESS);
        expect(proxiesBefore[0].length).toBeGreaterThanOrEqual(2);

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "removeProxies",
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const proxiesAfter = await context.polkadotJs().query.proxy.proxies(ALITH_ADDRESS);
        expect(proxiesAfter[0].isEmpty).toBe(true);
      },
    });

    it({
      id: "T07",
      title: "should fails if incorrect delay",
      test: async () => {
        const randomAccount = privateKeyToAccount(generatePrivateKey()).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [randomAccount, CONTRACT_PROXY_TYPE_STAKING, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        expect(
          await context.readPrecompile!({
            precompileName: "Proxy",
            functionName: "isProxy",
            args: [ALITH_ADDRESS, randomAccount, CONTRACT_PROXY_TYPE_STAKING, 2],
          })
        ).to.be.false;
      },
    });

    it({
      id: "T08",
      title: "should fails if incorrect proxyType",
      test: async () => {
        const randomAccount = privateKeyToAccount(generatePrivateKey()).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [randomAccount, CONTRACT_PROXY_TYPE_STAKING, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        expect(
          await context.readPrecompile!({
            precompileName: "Proxy",
            functionName: "isProxy",
            args: [ALITH_ADDRESS, randomAccount, CONTRACT_PROXY_TYPE_ANY, 0],
          })
        ).to.be.false;
      },
    });

    it({
      id: "T09",
      title: "should succeed if exists on read",
      test: async () => {
        const randomAccount = privateKeyToAccount(generatePrivateKey()).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [randomAccount, CONTRACT_PROXY_TYPE_STAKING, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        expect(
          await context.readPrecompile!({
            precompileName: "Proxy",
            functionName: "isProxy",
            args: [ALITH_ADDRESS, randomAccount, CONTRACT_PROXY_TYPE_STAKING, 0],
          })
        ).to.be.true;
      },
    });

    it({
      id: "T10",
      title: "shouldn't accept unknown proxy",
      test: async () => {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "proxy",
          args: [ALITH_ADDRESS, CHARLETH_ADDRESS, "0x00"],
          rawTxOnly: true,
          gas: 1_000_000n,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Revert");

        expect(
          async () =>
            await context.writePrecompile!({
              precompileName: "Proxy",
              functionName: "proxy",
              args: [ALITH_ADDRESS, CHARLETH_ADDRESS, "0x00"],
            })
        ).rejects.toThrowError("Not proxy");
      },
    });

    it({
      id: "T11",
      title: "should accept known proxy",
      test: async () => {
        const privateKey = generatePrivateKey();
        const randomAccount = privateKeyToAccount(privateKey).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const { abi } = fetchCompiledContract("Proxy");
        const rawTxn2 = await createViemTransaction(context, {
          to: PRECOMPILE_PROXY_ADDRESS,
          privateKey: BALTATHAR_PRIVATE_KEY,
          value: 1000n,
          data: encodeFunctionData({
            abi,
            functionName: "proxy",
            args: [ALITH_ADDRESS, randomAccount, "0x00"],
          }),
        });
        const { result: result2 } = await context.createBlock(rawTxn2);
        expectEVMResult(result2!.events, "Succeed");

        expect(await context.viem().getBalance({ address: randomAccount })).toBe(1000n);
      },
    });

    it({
      id: "T12",
      title: "shouldn't accept removed proxy",
      test: async () => {
        const privateKey = generatePrivateKey();
        const randomAccount = privateKeyToAccount(privateKey).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [CHARLETH_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const rawTxn2 = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "removeProxy",
          args: [CHARLETH_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          rawTxOnly: true,
        });
        const { result: result2 } = await context.createBlock(rawTxn2);
        expectEVMResult(result2!.events, "Succeed");

        const { abi } = fetchCompiledContract("Proxy");
        const rawTxn3 = await createViemTransaction(context, {
          to: PRECOMPILE_PROXY_ADDRESS,
          privateKey: CHARLETH_PRIVATE_KEY,
          value: 1000n,
          skipEstimation: true,
          data: encodeFunctionData({
            abi,
            functionName: "proxy",
            args: [ALITH_ADDRESS, randomAccount, "0x00"],
          }),
        });
        const { result: result3 } = await context.createBlock(rawTxn3);
        expectEVMResult(result3!.events, "Revert");

        expect(
          async () =>
            await createViemTransaction(context, {
              to: PRECOMPILE_PROXY_ADDRESS,
              privateKey: CHARLETH_PRIVATE_KEY,
              value: 1000n,
              data: encodeFunctionData({
                abi,
                functionName: "proxy",
                args: [ALITH_ADDRESS, randomAccount, "0x00"],
              }),
            })
        ).rejects.toThrowError("Not proxy");
      },
    });

    it({
      id: "T13",
      title: "shouldn't accept instant for delayed proxy",
      test: async () => {
        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [FAITH_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 2],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const { abi } = fetchCompiledContract("Proxy");
        const rawTxn2 = await createViemTransaction(context, {
          to: PRECOMPILE_PROXY_ADDRESS,
          privateKey: FAITH_PRIVATE_KEY,
          value: 1000n,
          skipEstimation: true,
          data: encodeFunctionData({
            abi,
            functionName: "proxy",
            args: [ALITH_ADDRESS, CHARLETH_ADDRESS, "0x00"],
          }),
        });
        const { result: result2 } = await context.createBlock(rawTxn2);
        expectEVMResult(result2!.events, "Revert");

        expect(
          async () =>
            await createViemTransaction(context, {
              to: PRECOMPILE_PROXY_ADDRESS,
              privateKey: FAITH_PRIVATE_KEY,
              value: 1000n,
              data: encodeFunctionData({
                abi,
                functionName: "proxy",
                args: [ALITH_ADDRESS, CHARLETH_ADDRESS, "0x00"],
              }),
            })
        ).rejects.toThrowError("Unannounced");
      },
    });

    it({
      id: "T14",
      title: "should transfer using value",
      test: async () => {
        const privateKey = generatePrivateKey();
        const randomAccount = privateKeyToAccount(privateKey).address;

        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [GOLIATH_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const contractBalBefore = await context
          .viem()
          .getBalance({ address: PRECOMPILE_PROXY_ADDRESS });
        const alithBalBefore = await context.viem().getBalance({ address: ALITH_ADDRESS });
        const { abi } = fetchCompiledContract("Proxy");
        const rawTxn2 = await createViemTransaction(context, {
          to: PRECOMPILE_PROXY_ADDRESS,
          privateKey: GOLIATH_PRIVATE_KEY,
          value: parseEther("5"),
          data: encodeFunctionData({
            abi,
            functionName: "proxy",
            args: [ALITH_ADDRESS, randomAccount, "0x00"],
          }),
        });
        const { result: result2 } = await context.createBlock(rawTxn2);
        expectEVMResult(result2!.events, "Succeed");

        expect(await context.viem().getBalance({ address: randomAccount })).toBe(parseEther("5"));
        const contractBalAfter = await context
          .viem()
          .getBalance({ address: PRECOMPILE_PROXY_ADDRESS });
        expect(contractBalBefore - contractBalAfter).to.equal(0n);
        const alithBalAfter = await context.viem().getBalance({ address: ALITH_ADDRESS });
        expect(alithBalBefore - alithBalAfter).to.equal(parseEther("5"));
      },
    });

    it({
      id: "T15",
      title: "should transfer using balances precompile",
      test: async () => {
        // The account cannot be random otherwise the calldata might contain more
        // zero bytes and have a different gas cost
        const randomAccount = "0x1ced798a66b803d0dbb665680283980a939a6432";
        // The tx can create an account, so record 148 bytes of storage growth
        // Storage growth ratio is 366
        // storage_gas = 148 * 366 = 54168
        const expectedMinimumPovGas = 59000;
        const rawTxn = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "addProxy",
          args: [BALTATHAR_ADDRESS, CONTRACT_PROXY_TYPE_ANY, 0],
          privateKey: FAITH_PRIVATE_KEY,
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTxn);
        expectEVMResult(result!.events, "Succeed");

        const balBefore = await context.viem().getBalance({ address: FAITH_ADDRESS });
        const { abi: ierc20Abi } = fetchCompiledContract("IERC20");

        const rawTxn2 = await context.writePrecompile!({
          precompileName: "Proxy",
          functionName: "proxy",
          args: [
            FAITH_ADDRESS,
            PRECOMPILE_NATIVE_ERC20_ADDRESS,
            encodeFunctionData({
              abi: ierc20Abi,
              functionName: "transfer",
              args: [randomAccount, parseEther("5")],
            }),
          ],
          privateKey: BALTATHAR_PRIVATE_KEY,
          rawTxOnly: true,
        });

        const { result: result2 } = await context.createBlock(rawTxn2);
        expectEVMResult(result2!.events, "Succeed");

        const { gasUsed } = await context
          .viem()
          .getTransactionReceipt({ hash: result2!.hash as `0x${string}` });

        // proof size reclaim seems indeterministic
        expect(gasUsed).toBeGreaterThan(expectedMinimumPovGas);
        expect(gasUsed).toBeLessThan(expectedMinimumPovGas + 2000);

        expect(await context.viem().getBalance({ address: randomAccount })).toBe(parseEther("5"));

        const balAfter = await context.viem().getBalance({ address: FAITH_ADDRESS });
        expect(balBefore - balAfter).to.equal(parseEther("5"));
      },
    });
  },
});
