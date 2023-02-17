import "@moonbeam-network/api-augment";
import { u8aToHex } from "@polkadot/util";
import { expect } from "chai";
import { ethers } from "ethers";
import { alith } from "../../util/accounts";
import { getObjectMethods } from "../../util/common";
import { GLMR } from "../../util/constants";

import { getCompiled } from "../../util/contracts";

import { expectEVMResult } from "../../util/eth-transactions";
import { web3EthCall } from "../../util/providers";

import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createContractExecution } from "../../util/transactions";

const REFERENDA_CONTRACT = getCompiled("Referenda");
const REFERENDA_INTERFACE = new ethers.utils.Interface(REFERENDA_CONTRACT.contract.abi);

describeDevMoonbeam("Precompiles - Referenda Auto Upgrade Demo", (context) => {
  it.only("should be accessible from a smart contract", async function () {
    const setStorageCallIndex = u8aToHex(context.polkadotApi.tx.system.setStorage.callIndex);
    let nonce = (await context.polkadotApi.rpc.system.accountNextIndex(alith.address)).toNumber();
    const contractV1 = await createContract(
      context,
      "ReferendaAutoUpgradeDemoV1",
      {
        nonce: nonce++,
      },
      ["root", setStorageCallIndex]
    );
    const contractV2 = await createContract(
      context,
      "ReferendaAutoUpgradeDemoV2",
      {
        nonce: nonce++,
      },
      ["root", setStorageCallIndex]
    );
    await context.createBlock([contractV1.rawTx, contractV2.rawTx]);

    const contractJson = getCompiled("ReferendaAutoUpgradeDemoV1");
    const contractAbi = new ethers.utils.Interface(contractJson.contract.abi);

    const ethersContract = new ethers.Contract(
      contractV1.contractAddress,
      contractAbi,
      context.ethers
    );

    expect(
      (await ethersContract.version()).toBigInt(),
      "Version should first be initialized to 1"
    ).to.equals(1n);

    const v1Code = await context.polkadotApi.query.evm.accountCodes(contractV1.contractAddress);
    const v2Code = await context.polkadotApi.query.evm.accountCodes(contractV2.contractAddress);
    const v1CodeKey = context.polkadotApi.query.evm.accountCodes.key(contractV1.contractAddress);
    const v2CodeKey = context.polkadotApi.query.evm.accountCodes.key(contractV2.contractAddress);
    const v1CodeStorage = (await context.polkadotApi.rpc.state.getStorage(v1CodeKey)) as any;
    const v2CodeStorage = (await context.polkadotApi.rpc.state.getStorage(v2CodeKey)) as any;

    // console.log("Create call");
    // const call = context.polkadotApi.tx.sudo.sudo(
    //   context.polkadotApi.tx.system.setStorage([[v1CodeKey, v2CodeStorage.toHex()]])
    // );
    // console.log("Create block");
    // console.log(`[${context.polkadotApi.tx.system.remark("test").toU8a().join(", ")}]`);
    // console.log(
    //   `[${context.polkadotApi.tx.referenda
    //     .submit(
    //       { system: "Root" },
    //       {
    //         Inline: context.polkadotApi.tx.system.remark("test").method.toHex(),
    //       },
    //       { After: 1 }
    //     )
    //     .toU8a()
    //     .join(", ")}]`
    // );
    // console.log(
    //   `[${context.polkadotApi.tx.referenda
    //     .submit(
    //       { system: "Root" },
    //       {
    //         Inline: context.polkadotApi.tx.system.setStorage([[v1CodeKey, v2CodeStorage.toHex()]]),
    //       },
    //       { After: 1 }
    //     )
    //     .toU8a()
    //     .join(", ")}]`
    // );
    // await context.createBlock(call);

    expect(await context.polkadotApi.query.evm.accountCodes(contractV1.contractAddress)).to.not.eq(
      v1Code
    );
    console.log(`     Address: ${contractV1.contractAddress}`);
    console.log(`         Key: ${v1CodeKey}`);
    console.log(`New Contract: ${v2CodeStorage.toHex()}`);

    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.balances.setBalance(contractV1.contractAddress, 5_000_000n * GLMR, 0)
      )
    );

    const data = await context.createBlock(
      createContractExecution(context, {
        contract: contractV1.contract,
        contractCall: contractV1.contract.methods.autoUpgrade(
          v2CodeStorage.toU8a().slice(1),
          v1CodeKey
        ),
      })
    );
    console.log(data);
    console.log("Result");
    expect(
      (await ethersContract.version()).toBigInt(),
      "Version should haven update to 2"
    ).to.equals(2n);
  });
});
