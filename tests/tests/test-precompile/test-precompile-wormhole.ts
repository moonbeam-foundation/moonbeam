import { PRECOMPILE_GMP_ADDRESS } from "../../util/constants";
import { expectEVMResult } from "../../util/eth-transactions";
import { describeDevMoonbeam } from "../../util/setup-dev-tests";
import { createContract, createTransaction } from "../../util/transactions";
import { getCompiled } from "../../util/contracts";
import { ethers } from "ethers";
import { expectSubstrateEvents } from "../../util/expect";
import { ALITH_ADDRESS, ALITH_PRIVATE_KEY } from "../../util/accounts";

const GMP_CONTRACT_JSON = getCompiled("GmpPrecompile");
const GMP_INTERFACE = new ethers.utils.Interface(GMP_CONTRACT_JSON.contract.abi);

describeDevMoonbeam(`Test local Wormhole`, (context) => {
  it("should support Alith VAA", async function () {
    this.timeout(3600 * 1000);

    const wormholeImplContract = await createContract(context, "WormholeImplementation", {
      gas: 12_500_000,
    });
    await context.createBlock(wormholeImplContract.rawTx);
    const wormholeContract = await createContract(
      context,
      "Wormhole",
      {
        gas: 12_500_000,
      },
      [
        wormholeImplContract.contractAddress,
        "0xb400c57a" +
          "0000000000000000000000000000000000000000000000000000000000000010" + // chainId
          "0000000000000000000000000000000000000000000000000000000000000001" + // GovernanceChainId
          "0000000000000000000000000000000000000000000000000000000000000004" + // GovernanceContract
          "0000000000000000000000000000000000000000000000000000000000000080" + // GuardianSet length
          "0000000000000000000000000000000000000000000000000000000000000040" + // Keys length (1)
          "0000000000000000000000000000000000000000000000000000000000000000" + // Guardian expiration
          "0000000000000000000000000000000000000000000000000000000000000001" + //
          "000000000000000000000000" +
          ALITH_ADDRESS.slice(2), // Guardian address
      ]
    );

    await context.createBlock(wormholeContract.rawTx);

    // TODO: remove this part once gmp precompile supports non hardcoded addresses

    const wormholeStorageKey = context.polkadotApi.query.evm.accountCodes.key(
      wormholeContract.contractAddress
    );
    const wormholeStorage = (await context.polkadotApi.rpc.state.getStorage(
      wormholeStorageKey
    )) as any;
    const hardCodedWormholeAddress = "0xa5B7D85a8f27dd7907dc8FdC21FA5657D5E2F901";
    console.log(wormholeContract.contractAddress);
    const hardCodedWormholeKey =
      context.polkadotApi.query.evm.accountCodes.key(hardCodedWormholeAddress);
    await context.createBlock(
      context.polkadotApi.tx.sudo.sudo(
        context.polkadotApi.tx.system.setStorage([[hardCodedWormholeKey, wormholeStorage.toHex()]])
      )
    );

    /**
      https://github.com/wormhole-foundation/wormhole/blob/main/ethereum/contracts/Messages.sol#L147
    **/

    const vaa_body =
      "640b665d" + // timestamp
      "00000001" + // nonce
      "000a" + // emitter chain
      "000000000000000000000000599cea2204b4faecd584ab1f2b6aca137a0afbe8" + // emitter address
      "000000000000009e" + // sequence
      "01" + // "consistency level"
      // PAYLOAD
      "030000000000000000000000000000000000000000000000000000000001312d" +
      "00000000000000000000000000f1277d1ed8ad466beddf92ef448a1326619566" +
      "21000a0000000000000000000000000000000000000000000000000000000000" +
      "0008150010000000000000000000000000b7e8c35609ca73277b2207d07b51c9" +
      "ac5798b380000000000000000000000000000000000000000000000000000000" +
      "0000000020000000000000000000000000000000000000000000000000000000" +
      "00000000807b2022706172656e7473223a20312c2022696e746572696f72223a" +
      "207b20225832223a205b207b202250617261636861696e223a20383838207d2c" +
      "207b20224163636f756e744b65793230223a2022307833353442313044343765" +
      "3834413030366239453765363641323239443137344538464632413036332220" +
      "7d205d7d7d";

    const signature = context.web3.eth.accounts.sign(vaa_body, ALITH_PRIVATE_KEY);

    const vaa_header =
      "0x01" + // version
      "00000000" + // guardian_set_index
      "01" + // signature count
      // SIGNATURE
      ("00" + // guardian index
        signature.r.slice(2) + // r
        signature.s.slice(2) + // S
        signature.v.slice(2)); // v

    const vaa = vaa_header + vaa_body;
    console.log(vaa);

    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [vaa]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 10_000_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Stopped");
    const evmEvents = expectSubstrateEvents(result, "evm", "Log");
  });
});
