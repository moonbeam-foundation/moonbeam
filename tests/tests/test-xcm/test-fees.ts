import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  BALTATHAR_ADDRESS,
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
} from "../../util/accounts";
import { expectOk } from "../../util/expect";
import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  CHARLETH_TRANSACTION_TEMPLATE,
  createContract,
  createTransaction,
  TRANSACTION_TEMPLATE,
} from "../../util/transactions";
import {
  CONTRACT_PROXY_TYPE_ANY,
  CONTRACT_PROXY_TYPE_GOVERNANCE,
  CONTRACT_PROXY_TYPE_STAKING,
  PRECOMPILE_PROXY_ADDRESS,
} from "../../util/constants";
import { expectEVMResult } from "../../util/eth-transactions";
import { web3EthCall } from "../../util/providers";
import { system } from "@substrate/txwrapper-substrate/lib/methods";
import Keyring from "@polkadot/keyring";
import { KeyringPair } from "@substrate/txwrapper-core";
import { SubmittableExtrinsic } from "@polkadot/api/promise/types";
import { ISubmittableResult } from "@polkadot/types/types";

const PROXY_CONTRACT_JSON = getCompiled("Looper");
const PROXY_INTERFACE = new ethers.utils.Interface(PROXY_CONTRACT_JSON.contract.abi);

// 10       753,350,000
// 100    2,209,100,000
// 1000  16,766,900,000
describeDevMoonbeam("Test Fee", (context) => {
  it("should foo", async function () {
    this.timeout(30_000_000);
    const contractAddr = "0xc01Ee7f10EA4aF4673cFff62710E1D7792aBa8f3";
    const keyring = new Keyring({ type: "ethereum" });
    const alith = keyring.addFromUri(ALITH_PRIVATE_KEY);
    const baltathar = keyring.addFromUri(BALTATHAR_PRIVATE_KEY);
    const charleth = keyring.addFromUri(CHARLETH_PRIVATE_KEY);
    //     const balanceBefore1 = await context.polkadotApi.query.system.account(ALITH_ADDRESS);
    //     await expectOk(context.createBlock(context.polkadotApi.tx.system.remark("foobar")));
    //     const balanceAfter1 = await context.polkadotApi.query.system.account(ALITH_ADDRESS);
    //     console.log(`
    // before: ${balanceBefore1.data.free.toString()}
    // after : ${balanceAfter1.data.free.toString()}
    // diff  : ${balanceBefore1.data.free.sub(balanceAfter1.data.free).toString()}`);
    // let baltatharNonce = await context.web3.eth.getTransactionCount(BALTATHAR_ADDRESS);
    // let charlethNonce = await context.web3.eth.getTransactionCount(CHARLETH_ADDRESS);
    const blockNumber = (
      await context.polkadotApi.rpc.chain.getBlock()
    ).block.header.number.toNumber();
    if (blockNumber === 0) {
      const { contract, rawTx } = await createContract(context, "Looper", {
        ...ALITH_TRANSACTION_TEMPLATE,
        gas: 5_000_000,
      });
      await expectOk(context.createBlock(rawTx));
      console.log("addr", contract.options.address);
    }

    const call = PROXY_INTERFACE.encodeFunctionData("incrementalLoop", ["1000"]);
    // await expectOk(
    //   context.createBlock(
    //     context.polkadotApi.tx.evm
    //       .call(charleth.address, contractAddr, call, 0, 900_000, 1_000_000_000n, null, null, [])
    //       .signAsync(charleth)
    //   )
    // );
    const { result } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        gas: 900_000,
        to: contractAddr, //contract.options.address,
        data: PROXY_INTERFACE.encodeFunctionData("infinite"),
        nonce: null,
      })
    );
    const receipt = await context.web3.eth.getTransactionReceipt(result.hash);
    expect(receipt.status).to.equal(true);

    return;

    const repsPerLoad = 20;
    for await (const loadFactor of [1, 10, 50, 100, 50, 10, 1, 1, 1, 1]) {
      for await (const rep of new Array(repsPerLoad).keys()) {
        // const loadFactor = 1;
        console.log(`load: ${loadFactor} (${rep + 1})`);
        const fees = await txObserveFeeDiff(context, async () => {
          const txs = [];
          let alithNonce = await context.web3.eth.getTransactionCount(ALITH_ADDRESS);
          let baltatharNonce = await context.web3.eth.getTransactionCount(BALTATHAR_ADDRESS);
          let charlethNonce = await context.web3.eth.getTransactionCount(CHARLETH_ADDRESS);
          for await (const _ of new Array(loadFactor)) {
            txs.push(
              await createTransaction(context, {
                ...ALITH_TRANSACTION_TEMPLATE,
                gas: 900_000,
                to: contractAddr, //contract.options.address,
                data: PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
                nonce: alithNonce++,
              })
            );
          }
          txs.push(
            await context.polkadotApi.tx.evm
              .call(
                baltathar.address,
                contractAddr,
                PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
                0,
                900_000,
                0n,
                null,
                null,
                // (baltatharNonce += 2).toString(),
                []
              )
              .signAsync(baltathar)
          );
          txs.push(
            await context.polkadotApi.tx.evm
              .call(
                charleth.address,
                contractAddr,
                PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
                0,
                900_000,
                1_000_000_000n,
                null,
                null,
                // (charlethNonce += 2).toString(),
                []
              )
              .signAsync(charleth)
          );

          return txs;
        });

        console.log(fees);
        // await new Promise((r) => setTimeout(r, 3000));
      }
    }

    //     for await (const loadFactor of [1, 10, 50, 100]) {
    //       // const loadFactor = 1;
    //       console.log(`load: ${loadFactor}`);
    //       const feeSubstrate = await txObserveFees(context, contractAddr, baltathar, () => {
    //         const txs = [];
    //         for (const _ of new Array(loadFactor)) {
    //           console.log("bnonce", baltatharNonce);
    //           txs.push(
    //             context.polkadotApi.tx.evm
    //               .call(
    //                 baltathar.address,
    //                 contractAddr,
    //                 PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
    //                 0,
    //                 900_000,
    //                 0n,
    //                 null,
    //                 (baltatharNonce++).toString(),
    //                 []
    //               )
    //               .signAsync(baltathar)
    //           );
    //         }

    //         return txs;
    //       });
    //       const feeEthereum = await txObserveFees(context, contractAddr, charleth, () => {
    //         const txs = [];
    //         for (const _ of new Array(loadFactor)) {
    //           console.log("cnonce", charlethNonce);
    //           txs.push(
    //             context.polkadotApi.tx.evm
    //               .call(
    //                 charleth.address,
    //                 contractAddr,
    //                 PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
    //                 0,
    //                 900_000,
    //                 1_000_000_000n,
    //                 null,
    //                 (charlethNonce++).toString(),
    //                 []
    //               )
    //               .signAsync(charleth)
    //           );
    //         }
    //         return txs;
    //       });

    //       console.log(`substrate: ${feeSubstrate.diff.toString()}
    // ethereum : ${feeEthereum.diff.toString()}`);
    //       await new Promise((r) => setTimeout(r, 3000));
    //     }
    // const balanceBefore = await context.polkadotApi.query.system.account(BALTATHAR_ADDRESS);
    // const {
    //   result: { events },
    // } = await context.createBlock(
    //   context.polkadotApi.tx.evm
    //     .call(
    //       BALTATHAR_ADDRESS,
    //       contractAddr,
    //       PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
    //       0,
    //       900_000,
    //       // 1_000_000_000n,
    //       0n,
    //       null,
    //       undefined,
    //       []
    //     )
    //     .signAsync(baltathar)
    // );
    // const {
    //   result: { events },
    // } = await context.createBlock(
    //   context.polkadotApi.tx.sudo.sudo(
    //     context.polkadotApi.tx.evm.call(
    //       alith.address,
    //       "0x3ed62137c5DB927cb137c26455969116BF0c23Cb",
    //       PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
    //       0,
    //       900_000,
    //       1_000_000_000n,
    //       "0",
    //       undefined,
    //       []
    //     )
    //   )
    // );

    // let aliceNonce = await context.web3.eth.getTransactionCount(ALITH_ADDRESS);
    // const txs = [];
    // for await (const _ of new Array(1)) {
    //   txs.push(
    //     await createTransaction(context, {
    //       ...ALITH_TRANSACTION_TEMPLATE,
    //       gas: 900_000,
    //       to: "0x3ed62137c5DB927cb137c26455969116BF0c23Cb", //contract.options.address,
    //       data: PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
    //       nonce: aliceNonce++,
    //     })
    //   );
    // }
    // await context.createBlock(txs);
    // await expectOk(context.createBlock(txs));
    //     const balanceAfter = await context.polkadotApi.query.system.account(ALITH_ADDRESS);
    //     console.log(`
    // before: ${balanceBefore.data.free.toString()}
    // after : ${balanceAfter.data.free.toString()}
    // diff  : ${balanceBefore.data.free.sub(balanceAfter.data.free).toString()}`);
    // expectEVMResult(result.events, "Succeed");

    // console.log(result.extrinsic.toHuman());
    // result.events.forEach((e) => console.log(e.toHuman()));
  });
});

async function txObserveFees(
  context: DevTestContext,
  contractAddr: string,
  sender: KeyringPair,
  txFunc: () => SubmittableExtrinsic[]
) {
  const balanceBefore = await context.polkadotApi.query.system.account(sender.address);
  const txs = txFunc();
  console.log(txs);
  const { result } = await context.createBlock(
    // context.polkadotApi.tx.evm
    //   .call(
    //     sender.address,
    //     contractAddr,
    //     PROXY_INTERFACE.encodeFunctionData("incrementalLoop", [1000]),
    //     0,
    //     900_000,
    //     // 1_000_000_000n,
    //     sender.address === BALTATHAR_ADDRESS ? 0n : 1_000_000_000n,
    //     null,
    //     undefined,
    //     []
    //   )
    // .signAsync(sender)
    txs
  );
  const balanceAfter = await context.polkadotApi.query.system.account(sender.address);
  // console.log(`
  // RESULT: ${result.successful}
  // sender: ${sender.address.toString()}
  // before: ${balanceBefore.data.free.toString()}
  // after : ${balanceAfter.data.free.toString()}
  // diff  : ${balanceBefore.data.free.sub(balanceAfter.data.free).toString()}`);

  return {
    before: balanceBefore.data.free,
    after: balanceAfter.data.free,
    diff: balanceBefore.data.free.sub(balanceAfter.data.free),
  };
}

async function txObserveFeeDiff(
  context: DevTestContext,
  txFunc: () => Promise<SubmittableExtrinsic[]>
) {
  const txs = await txFunc();
  // console.log(txs);
  const balanceBeforeBaltathar = await context.polkadotApi.query.system.account(BALTATHAR_ADDRESS);
  const balanceBeforeCharleth = await context.polkadotApi.query.system.account(CHARLETH_ADDRESS);
  // await expectOk(context.createBlock(txs));
  await context.createBlock(txs);
  const balanceAfterBaltathar = await context.polkadotApi.query.system.account(BALTATHAR_ADDRESS);
  const balanceAfterCharleth = await context.polkadotApi.query.system.account(CHARLETH_ADDRESS);

  return {
    substrate: balanceBeforeBaltathar.data.free.sub(balanceAfterBaltathar.data.free).toString(),
    evm: balanceBeforeCharleth.data.free.sub(balanceAfterCharleth.data.free).toString(),
  };
}
