import { describeDevMoonbeam, DevTestContext } from "../../util/setup-dev-tests";
import {
  createContract,
  createContractExecution,
  createTransaction,
} from "../../util/transactions";
import { getCompiled } from "../../util/contracts";
import {
  genRegisterChainVAA,
  genAssetMeta,
  genTransferVAA,
  genTransferWithPayloadVAA,
} from "../../util/wormhole";
import { ethers } from "ethers";
import { alith, ALITH_ADDRESS, ALITH_PRIVATE_KEY, BALTATHAR_ADDRESS } from "../../util/accounts";
import { PRECOMPILE_GMP_ADDRESS } from "../../util/constants";
import { expectSubstrateEvent, expectSubstrateEvents } from "../../util/expect";

import { expectEVMResult } from "../../util/eth-transactions";
const debug = require("debug")("test:wormhole");

const GUARDIAN_SET_INDEX = 0;

const GMP_CONTRACT_JSON = getCompiled("precompiles/gmp/Gmp");
const GMP_INTERFACE = new ethers.utils.Interface(GMP_CONTRACT_JSON.contract.abi);

const WORMHOLE_CONTRACT_JSON = getCompiled("wormhole/Wormhole");
const WORMHOLE_INTERFACE = new ethers.utils.Interface(WORMHOLE_CONTRACT_JSON.contract.abi);

const TOKEN_BRIDGE_IMPL_CONTRACT_JSON = getCompiled("wormhole/bridge/token/TokenImplementation");
const TOKEN_BRIDGE_IMPL_INTERFACE = new ethers.utils.Interface(
  TOKEN_BRIDGE_IMPL_CONTRACT_JSON.contract.abi
);

const TOKEN_BRIDGE_CONTRACT_JSON = getCompiled("wormhole/bridge/token/BridgeToken");
const TOKEN_BRIDGE_INTERFACE = new ethers.utils.Interface(TOKEN_BRIDGE_CONTRACT_JSON.contract.abi);

/*
  Alphanet 2023-03-17

  Wormhole: 0xa5B7D85a8f27dd7907dc8FdC21FA5657D5E2F901
  Wormhole Impl: 0x99737ec4b815d816c49a385943baf0380e75c0ac
  ChainId: 16 
  EvmChainId: 1287
  GovernanceChainId: 1
  GovernanceContract: 0x0000000000000000000000000000000000000000000000000000000000000004

  TokenBridge: 0xbc976D4b9D57E57c3cA52e1Fd136C45FF7955A96
  TokenBridge Impl: 0x430855b4d43b8aeb9d2b9869b74d58dda79c0db2
  WETH: 0xd909178cc99d318e4d46e7e66a972955859670e1
  ChainId: 16 
  EvmChainId: 1287
  Finality: 1
  Implementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 
  Wormhole: 0xa5b7d85a8f27dd7907dc8fdc21fa5657d5e2f901
  TokenImplementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 

  TokenImplementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 
*/

const deploy = async (context: DevTestContext, contractPath: string, initData?: any[]) => {
  const contract = await createContract(context, contractPath, {}, initData);
  const result = await context.createBlock(contract.rawTx);
  debug(
    `Created ${contractPath}: ${contract.contractAddress} => ${result.result.hash} (${
      result.result.error || "good"
    })`
  );
  return contract;
};

describeDevMoonbeam(`Test local Wormhole`, (context) => {
  it("should support Alith VAA", async function () {
    this.timeout(3600 * 1000);

    const wethContract = await deploy(context, "wormhole/bridge/mock/MockWETH9");
    debug(`weth contract deployed to ${wethContract.contractAddress}`);
    const myTokenContract = await deploy(context, "wormhole/bridge/mock/MockWETH9");

    const initialSigners = [ALITH_ADDRESS];
    const signerPKs = [ALITH_PRIVATE_KEY];
    const chainId = "0x10";
    const governanceChainId = "0x1";
    const governanceContract = "0x0000000000000000000000000000000000000000000000000000000000000004";
    const evmChainId = await context.web3.eth.getChainId(); //"1337"; // "1281";
    // Deploy wormhole (based on wormhole)
    // wormhole-foundation/wormhole/blob/main/ethereum/migrations/2_deploy_wormhole.js
    const setupContract = await deploy(context, "wormhole/Setup");
    const implementationContract = await deploy(context, "wormhole/Implementation");
    const wormholeSetupData = setupContract.contract.methods
      .setup(
        implementationContract.contractAddress,
        initialSigners,
        evmChainId,
        governanceChainId,
        governanceContract,
        evmChainId
      )
      .encodeABI();
    const wormholeContract = await deploy(context, "wormhole/Wormhole", [
      setupContract.contractAddress,
      wormholeSetupData,
    ]);

    debug(`wormhole core bridge deployed to ${wormholeContract.contractAddress}`);

    const finality = 1;
    // Deploy bridge (based on wormhole)
    // wormhole-foundation/wormhole/blob/main/ethereum/migrations/3_deploy_bridge.js
    const tokenImplContract = await deploy(context, "wormhole/bridge/token/TokenImplementation");
    debug(`wormhole token impl deployed to ${tokenImplContract.contractAddress}`);
    const bridgeSetupContract = await deploy(context, "wormhole/bridge/BridgeSetup");
    const bridgeImplContract = await deploy(context, "wormhole/bridge/BridgeImplementation");
    const bridgeSetupData = bridgeSetupContract.contract.methods
      .setup(
        bridgeImplContract.contractAddress,
        evmChainId,
        wormholeContract.contractAddress,
        governanceChainId,
        governanceContract,
        tokenImplContract.contractAddress,
        wethContract.contractAddress,
        finality,
        evmChainId
      )
      .encodeABI();
    const bridgeContract = await deploy(context, "wormhole/bridge/TokenBridge", [
      bridgeSetupContract.contractAddress,
      bridgeSetupData,
    ]);

    debug(`bridge contract deployed to ${bridgeContract.contractAddress}`);

    const ETHEmitter = "0x0000000000000000000000003ee18b2214aff97000d974cf647e7c347e8fa585";
    const ETHChain = 3;
    let nonce = 0;

    // Register Chain ETH
    const registerChainVm = await genRegisterChainVAA(
      signerPKs,
      ETHEmitter,
      GUARDIAN_SET_INDEX,
      nonce++,
      1,
      ETHChain
    );
    const registerChainResult = await context.createBlock(
      createContractExecution(context, {
        contract: bridgeContract.contract,
        contractCall: bridgeImplContract.contract.methods.registerChain(`0x${registerChainVm}`),
      })
    );
    debug(
      `Registered chain ${ETHChain}: ${ETHEmitter} => ${registerChainResult.result.hash} (${
        registerChainResult.result.error || "good"
      })`
    );

    // Register Asset MyToken
    const assetMetaVm = await genAssetMeta(
      signerPKs,
      GUARDIAN_SET_INDEX,
      nonce++,
      1,
      wethContract.contractAddress,
      ETHChain,
      ETHEmitter,
      18,
      "WETH",
      "Wrapped Ether"
    );
    const assetMetaResult = await context.createBlock(
      createContractExecution(context, {
        contract: bridgeContract.contract,
        contractCall: bridgeImplContract.contract.methods.createWrapped(`0x${assetMetaVm}`),
      })
    );
    const wrappedToken = (await context.web3.eth.getTransactionReceipt(assetMetaResult.result.hash))
      .logs[0].address;
    debug(
      `Created Wrapped Asset ${wrappedToken} => ${assetMetaResult.result.hash} (${
        assetMetaResult.result.error || "good"
      })`
    );

    debug(`wrapped token deployed to ${wrappedToken}`);

    // before interacting with the precompile, we need to set some contract addresses from our
    // our deployments above
    const CORE_CONTRACT_STORAGE_ADDRESS =
      "0xb7f047395bba5df0367b45771c00de5059ff23ff65cc809711800d9d04e4b14c";
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [CORE_CONTRACT_STORAGE_ADDRESS, wormholeContract.contractAddress],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    const BRIDGE_CONTRACT_STORAGE_ADDRESS =
      "0xb7f047395bba5df0367b45771c00de50c1586bde54b249fb7f521faf831ade45";
    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [BRIDGE_CONTRACT_STORAGE_ADDRESS, bridgeContract.contractAddress],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    const transferVAA = await genTransferWithPayloadVAA(
      signerPKs,
      GUARDIAN_SET_INDEX,
      nonce++,
      123, // sequence
      999, // amount of tokens
      wethContract.contractAddress,
      ETHChain,
      ETHChain,
      ETHEmitter, // TODO: review
      PRECOMPILE_GMP_ADDRESS,
      "0x" + evmChainId.toString(16),
      "0x0000000000000000000000000000000000000001", // TODO: fromAddress
      "0x00010101000000000000000000000000000000000000000000000000000000000000000000"
    );

    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [`0x${transferVAA}`]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 500_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Returned");
    expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
  });
});
