import "@moonbeam-network/api-augment";
import { describeSuite, expect, beforeAll, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  CONTRACT_PROXY_TYPE_ANY,
  CONTRACT_PROXY_TYPE_STAKING,
  DOROTHY_ADDRESS,
  FAITH_ADDRESS,
  FAITH_PRIVATE_KEY,
  GLMR,
  GOLIATH_ADDRESS,
  PRECOMPILE_PROXY_ADDRESS,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { encodeFunctionData } from "viem";

describeSuite({
  id: "D2550",
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
          context.polkadotJs().tx.balances.transfer(randomAccount.address, GLMR)
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
          args: [ALITH_ADDRESS, CHARLETH_ADDRESS, []],
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
              args: [ALITH_ADDRESS, CHARLETH_ADDRESS, []],
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
            args: [ALITH_ADDRESS, randomAccount, []],
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
            args: [ALITH_ADDRESS, randomAccount, []],
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
              args: [ALITH_ADDRESS, randomAccount, []],
            }),
          })
        ).rejects.toThrowError("Not proxy");
      },
    });
  },
});


// describeSuite({id:"", title:"Pallet proxy - shouldn't accept instant for delayed proxy", foundationMethods:"dev",testCases: ({it, log, context}) => {
//   it({id:"T0",title:"shouldn't accept instant for delayed proxy", test:async () => {
//     const beforeCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
//     const {
//       result: { events },
//     } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_PROXY_ADDRESS,
//         data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
//           BALTATHAR_ADDRESS,
//           CONTRACT_PROXY_TYPE_ANY,
//           2,
//         ]),
//       })
//     );
//     expectEVMResult(events, "Succeed");

//     const {
//       result: { events: events2, hash: hash2 },
//     } = await context.createBlock(
//       createTransaction(context, {
//         ...BALTATHAR_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_PROXY_ADDRESS,
//         data: PROXY_INTERFACE.encodeFunctionData("proxy", [ALITH_ADDRESS, CHARLETH_ADDRESS, []]),
//         value: "0x64",
//       })
//     );
//     expectEVMResult(events2, "Revert");
//     const revertReason = await extractRevertReason(hash2, context.ethers);
//     expect(revertReason).to.contain("Unannounced");
//     const afterCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
//     expect(afterCharlethBalance - beforeCharlethBalance).to.be.eq(0n);
//   });
// });

// describeSuite({id:"", title:"Pallet proxy - should transfer using value", foundationMethods:"dev",testCases: ({it, log, context}) => {
//   it({id:"T0",title:"should transfer using value", test:async () => {
//     const {
//       result: { events },
//     } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_PROXY_ADDRESS,
//         data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
//           BALTATHAR_ADDRESS,
//           CONTRACT_PROXY_TYPE_ANY,
//           0,
//         ]),
//       })
//     );
//     expectEVMResult(events, "Succeed");

//     const beforeAlithBalance = BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS));
//     const beforeCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
//     const value = BigInt(context.web3.utils.toWei("10", "ether"));

//     const {
//       result: { events: events2, hash: hash2 },
//     } = await context.createBlock(
//       createTransaction(context, {
//         ...BALTATHAR_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_PROXY_ADDRESS,
//         data: PROXY_INTERFACE.encodeFunctionData("proxy", [ALITH_ADDRESS, CHARLETH_ADDRESS, []]),
//         value: value.toString(),
//       })
//     );

//     expectEVMResult(events2, "Succeed");

//     const { gasUsed } = await context.web3.eth.getTransactionReceipt(hash2);
//     expect(gasUsed).to.equal(40892);

//     const afterAlithBalance = BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS));
//     const afterCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
//     const afterProxyPrecompileBalance = BigInt(
//       await context.web3.eth.getBalance(PRECOMPILE_PROXY_ADDRESS)
//     );

//     expect(beforeAlithBalance - afterAlithBalance).to.equal(value);
//     expect(afterCharlethBalance - beforeCharlethBalance).to.equal(value);
//     expect(afterProxyPrecompileBalance).to.equal(0n);
//   });
// });

// describeSuite({id:"", title:"Pallet proxy - should transfer using balances precompile", foundationMethods:"dev",testCases: ({it, log, context}) => {
//   it({id:"T0",title:"should transfer using balances precompile", test:async () => {
//     const NATIVE_ERC20_CONTRACT = getCompiled("precompiles/balances-erc20/IERC20");
//     const NATIVE_ERC20_INTERFACE = new ethers.utils.Interface(NATIVE_ERC20_CONTRACT.contract.abi);

//     const {
//       result: { events },
//     } = await context.createBlock(
//       createTransaction(context, {
//         ...ALITH_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_PROXY_ADDRESS,
//         data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
//           BALTATHAR_ADDRESS,
//           CONTRACT_PROXY_TYPE_ANY,
//           0,
//         ]),
//       })
//     );
//     expectEVMResult(events, "Succeed");

//     const beforeAlithBalance = BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS));
//     const beforeCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
//     const value = BigInt(context.web3.utils.toWei("10", "ether"));

//     const {
//       result: { events: events2, hash: hash2 },
//     } = await context.createBlock(
//       createTransaction(context, {
//         ...BALTATHAR_TRANSACTION_TEMPLATE,
//         to: PRECOMPILE_PROXY_ADDRESS,
//         data: PROXY_INTERFACE.encodeFunctionData("proxy", [
//           ALITH_ADDRESS,
//           PRECOMPILE_NATIVE_ERC20_ADDRESS,
//           NATIVE_ERC20_INTERFACE.encodeFunctionData("transfer", [CHARLETH_ADDRESS, value]),
//         ]),
//       })
//     );

//     expectEVMResult(events2, "Succeed");

//     const { gasUsed } = await context.web3.eth.getTransactionReceipt(hash2);
//     expect(gasUsed).to.equal(33997);

//     const afterAlithBalance = BigInt(await context.web3.eth.getBalance(ALITH_ADDRESS));
//     const afterCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));

//     expect(beforeAlithBalance - afterAlithBalance).to.equal(value);
//     expect(afterCharlethBalance - beforeCharlethBalance).to.equal(value);
//   });
// });
