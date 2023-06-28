import "@polkadot/api-augment";
import "@moonbeam-network/api-augment";
import { expect } from "chai";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  BALTATHAR_SESSION_ADDRESS,
  CHARLETH_ADDRESS,
} from "../../util/accounts";

import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";
import {
  ALITH_TRANSACTION_TEMPLATE,
  BALTATHAR_TRANSACTION_TEMPLATE,
  createTransaction,
} from "../../util/transactions";
import {
  CONTRACT_PROXY_TYPE_NON_TRANSFER,
  PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
  PRECOMPILE_PROXY_ADDRESS,
} from "../../util/constants";
import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";

const AUTHOR_MAPPING_CONTRACT = getCompiled("precompiles/author-mapping/AuthorMapping");
const AUTHOR_MAPPING_INTERFACE = new ethers.utils.Interface(AUTHOR_MAPPING_CONTRACT.contract.abi);
const PROXY_CONTRACT_JSON = getCompiled("precompiles/proxy/Proxy");
const PROXY_INTERFACE = new ethers.utils.Interface(PROXY_CONTRACT_JSON.contract.abi);

export async function getMappingInfo(
  context: DevTestContext,
  authorId: string
): Promise<{ account: string; deposit: BigInt }> {
  const mapping = await context.polkadotApi.query.authorMapping.mappingWithDeposit(authorId);
  if (mapping.isSome) {
    return {
      account: mapping.unwrap().account.toString(),
      deposit: mapping.unwrap().deposit.toBigInt(),
    };
  }
  return null;
}

describeDevMoonbeam("Proxy : Non transfer - Evm transfer", (context) => {
  before("first add proxy", async () => {
    const {
      result: { events },
    } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_NON_TRANSFER,
          0,
        ]),
      })
    );
    expectEVMResult(events, "Succeed");
  });

  it("should fail in simple evm transfer", async function () {
    const beforeCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
    const {
      result: { events, hash },
    } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("proxy", [ALITH_ADDRESS, CHARLETH_ADDRESS, []]),
        value: 100,
      })
    );
    expectEVMResult(events, "Revert");
    const revertReason = await extractRevertReason(hash, context.ethers);
    expect(revertReason).to.contain("CallFiltered");

    const afterCharlethBalance = BigInt(await context.web3.eth.getBalance(CHARLETH_ADDRESS));
    expect(afterCharlethBalance).to.be.eq(beforeCharlethBalance);
  });
});

describeDevMoonbeam("Proxy : Non transfer - AuthorMapping", (context) => {
  it("should succeed in adding an association", async function () {
    const {
      result: { events },
    } = await context.createBlock(
      createTransaction(context, {
        ...ALITH_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("addProxy", [
          BALTATHAR_ADDRESS,
          CONTRACT_PROXY_TYPE_NON_TRANSFER,
          0,
        ]),
      })
    );
    expectEVMResult(events, "Succeed");

    const {
      result: { events: events2 },
    } = await context.createBlock(
      createTransaction(context, {
        ...BALTATHAR_TRANSACTION_TEMPLATE,
        to: PRECOMPILE_PROXY_ADDRESS,
        data: PROXY_INTERFACE.encodeFunctionData("proxy", [
          ALITH_ADDRESS,
          PRECOMPILE_AUTHOR_MAPPING_ADDRESS,
          AUTHOR_MAPPING_INTERFACE.encodeFunctionData("addAssociation", [
            BALTATHAR_SESSION_ADDRESS,
          ]),
        ]),
      })
    );
    expectEVMResult(events2, "Succeed");

    // // check association
    expect((await getMappingInfo(context, BALTATHAR_SESSION_ADDRESS)).account).to.eq(ALITH_ADDRESS);
  });
});
