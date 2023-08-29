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
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";

import { TypeRegistry, Enum, Struct } from "@polkadot/types";
import { expectEVMResult, extractRevertReason } from "../../util/eth-transactions";
import { expect } from "chai";

const debug = require("debug")("test:wormhole");

const GUARDIAN_SET_INDEX = 0;

// wormhole internally "compacts" amounts. they don't use a constant (it's more complicated) but we
// can get away with a constant here.
// TODO: maybe remove the implicit behavior from the util functions in wormhole.ts?
// TODO: actually, something is wrong here -- I don't think this matches the WH compacting logic
const WH_IMPLICIT_DECIMALS = 18n;
const WH_IMPLICIT_MULTIPLIER = 10n ** WH_IMPLICIT_DECIMALS;

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
  const signerPKs = [ALITH_PRIVATE_KEY];
  const ETHChain = 3;
  const ETHEmitter = "0x0000000000000000000000003ee18b2214aff97000d974cf647e7c347e8fa585";

  let whNonce = 0;
  // TODO: ugh, clean this up. we don't need the WETH contract we deployed, we need the wrapped
  // version of it created by WH.
  let wethAddress: string;
  let whWethContract: ethers.Contract;

  let gmpContract: ethers.Contract;

  let whWethAddress: string;
  let evmChainId;

  // destination used for xtoken transfers
  const versionedMultiLocation = {
    v1: {
      parents: 1,
      interior: {
        X1: {
          AccountKey20: {
            id: "0x0000000000000000000000000000000000000000000000000000000000000000",
          },
        },
      },
    },
  };

  before("deploy wormhole infrastructure", async function () {
    const wethDeployment = await deploy(context, "wormhole/bridge/mock/MockWETH9");
    // wethContract = wethDeployment.contract;
    wethAddress = wethDeployment.contractAddress;
    debug(`weth contract deployed to ${wethAddress}`);
    const myTokenContract = await deploy(context, "wormhole/bridge/mock/MockWETH9");

    const initialSigners = [ALITH_ADDRESS];
    const chainId = "0x10";
    const governanceChainId = "0x1";
    const governanceContract = "0x0000000000000000000000000000000000000000000000000000000000000004";
    evmChainId = await context.web3.eth.getChainId();
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
        wethAddress,
        finality,
        evmChainId
      )
      .encodeABI();
    const bridgeContract = await deploy(context, "wormhole/bridge/TokenBridge", [
      bridgeSetupContract.contractAddress,
      bridgeSetupData,
    ]);

    debug(`bridge contract deployed to ${bridgeContract.contractAddress}`);

    // Register Chain ETH
    const registerChainVm = await genRegisterChainVAA(
      signerPKs,
      ETHEmitter,
      GUARDIAN_SET_INDEX,
      whNonce++,
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
      whNonce++,
      1,
      wethAddress,
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

    // TODO: clean up / avoid using both web3js and ethers
    // TODO: not the right contract, but it'll probably work
    const WETH_CONTRACT_JSON = getCompiled("wormhole/bridge/mock/MockWETH9");
    const WETH_INTERFACE = new ethers.utils.Interface(WETH_CONTRACT_JSON.contract.abi);
    whWethContract = new ethers.Contract(wrappedToken, WETH_INTERFACE, context.ethers);

    debug(`wrapped token deployed to ${wrappedToken}`);

    // before interacting with the precompile, we need to set some contract addresses from our
    // our deployments above
    const CORE_CONTRACT_STORAGE_ADDRESS = u8aToHex(
      u8aConcat(xxhashAsU8a("gmp", 128), xxhashAsU8a("CoreAddress", 128))
    );
    expect(CORE_CONTRACT_STORAGE_ADDRESS).to.eq(
      "0xb7f047395bba5df0367b45771c00de5059ff23ff65cc809711800d9d04e4b14c"
    );

    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [CORE_CONTRACT_STORAGE_ADDRESS, wormholeContract.contractAddress],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    const BRIDGE_CONTRACT_STORAGE_ADDRESS = u8aToHex(
      u8aConcat(xxhashAsU8a("gmp", 128), xxhashAsU8a("BridgeAddress", 128))
    );
    expect(BRIDGE_CONTRACT_STORAGE_ADDRESS).to.eq(
      "0xb7f047395bba5df0367b45771c00de50c1586bde54b249fb7f521faf831ade45"
    );

    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [BRIDGE_CONTRACT_STORAGE_ADDRESS, bridgeContract.contractAddress],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    // we also need to disable the killswitch by setting the 'enabled' flag to Some(true)
    const ENABLED_FLAG_STORAGE_ADDRESS = u8aToHex(
      u8aConcat(xxhashAsU8a("gmp", 128), xxhashAsU8a("PrecompileEnabled", 128))
    );
    expect(ENABLED_FLAG_STORAGE_ADDRESS).to.eq(
      "0xb7f047395bba5df0367b45771c00de502551bba17abb82ef3498bab688e470b8"
    );

    await context.polkadotApi.tx.sudo
      .sudo(
        context.polkadotApi.tx.system.setStorage([
          [
            ENABLED_FLAG_STORAGE_ADDRESS,
            context.polkadotApi.registry.createType("Option<bool>", true).toHex(),
          ],
        ])
      )
      .signAndSend(alith);
    await context.createBlock();

    const gmpJson = getCompiled("wormhole/bridge/mock/MockWETH9");
    const gmpInterface = new ethers.utils.Interface(WETH_CONTRACT_JSON.contract.abi);
    gmpContract = new ethers.Contract(PRECOMPILE_GMP_ADDRESS, gmpInterface, context.ethers);
  });

  it("should support V1 user action", async function () {
    this.timeout(20000);

    // create payload
    const destination = context.polkadotApi.registry.createType(
      "VersionedMultiLocation",
      versionedMultiLocation
    );

    const userAction = new XcmRoutingUserAction({ destination });
    const versionedUserAction = new VersionedUserAction({ V1: userAction });
    let payload = "" + versionedUserAction.toHex();

    const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

    const whAmount = 999n;
    const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;

    const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);
    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [`0x${transferVAA}`]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 600_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Returned");
    const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
    const transferFungible = events[0].data[1][0].fun;
    expect(transferFungible.isFungible);
    const transferAmount = transferFungible.asFungible.toBigInt();
    expect(transferAmount).to.eq(realAmount);
  });

  it("should support V2 user action with fee", async function () {
    this.timeout(20000);

    // create payload
    const destination = context.polkadotApi.registry.createType(
      "VersionedMultiLocation",
      versionedMultiLocation
    );

    const whAmount = 999n;
    const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;
    const fee = 1234500n;

    const userAction = new XcmRoutingUserActionWithFee({ destination, fee });
    const versionedUserAction = new VersionedUserAction({ V2: userAction });

    const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

    const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);
    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [`0x${transferVAA}`]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 600_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Returned");
    const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
    const transferFungible = events[0].data[1][0].fun;
    expect(transferFungible.isFungible);
    const transferAmount = transferFungible.asFungible.toBigInt();
    expect(transferAmount).to.eq(realAmount - fee);

    const alithWHTokenAfter = await whWethContract.balanceOf(ALITH_ADDRESS);
    expect(alithWHTokenAfter - alithWHTokenBefore).to.eq(Number(fee));
  });

  it("should pay entire transfer when fee greater than transfer", async function () {
    this.timeout(20000);

    // create payload
    const destination = context.polkadotApi.registry.createType(
      "VersionedMultiLocation",
      versionedMultiLocation
    );

    const whAmount = 100n;
    const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;
    const fee = realAmount + 1n;

    const userAction = new XcmRoutingUserActionWithFee({ destination, fee });
    const versionedUserAction = new VersionedUserAction({ V2: userAction });

    const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

    const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);
    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [`0x${transferVAA}`]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 600_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Returned");
    // there should be no xTokens TransferredMultiAssets event since fee >= amount sent
    const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
    expect(events.length).to.eq(0); // TODO: isn't expectSubstrateEvents supposed to expect > 0?

    const alithWHTokenAfter = await whWethContract.balanceOf(ALITH_ADDRESS);
    expect(alithWHTokenAfter - alithWHTokenBefore).to.eq(Number(realAmount));
  });

  it("should pay no fee if fee is zero", async function () {
    this.timeout(20000);

    // create payload
    const destination = context.polkadotApi.registry.createType(
      "VersionedMultiLocation",
      versionedMultiLocation
    );

    const whAmount = 100n;
    const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;
    const fee = 0n;

    const userAction = new XcmRoutingUserActionWithFee({ destination, fee });
    const versionedUserAction = new VersionedUserAction({ V2: userAction });

    const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

    const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);
    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [`0x${transferVAA}`]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 600_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Succeed", "Returned");
    const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
    const transferFungible = events[0].data[1][0].fun;
    expect(transferFungible.isFungible);
    const transferAmount = transferFungible.asFungible.toBigInt();
    expect(transferAmount).to.eq(realAmount);

    // no fee paid
    const alithWHTokenAfter = await whWethContract.balanceOf(ALITH_ADDRESS);
    expect(alithWHTokenAfter - alithWHTokenBefore).to.eq(0);
  });

  async function makeTestVAA(amount: number, action: VersionedUserAction): Promise<string> {
    let payload = "" + action.toHex();

    return await genTransferWithPayloadVAA(
      signerPKs,
      GUARDIAN_SET_INDEX,
      whNonce++,
      123, // sequence
      amount,
      wethAddress,
      ETHChain,
      ETHChain,
      ETHEmitter, // TODO: review
      PRECOMPILE_GMP_ADDRESS,
      "0x" + evmChainId.toString(16),
      "0x0000000000000000000000000000000000000001", // TODO: fromAddress
      "" + payload
    );
  }
});

const registry = new TypeRegistry();

class VersionedUserAction extends Enum {
  constructor(value?: any) {
    super(registry, { V1: XcmRoutingUserAction, V2: XcmRoutingUserActionWithFee }, value);
  }
}
class XcmRoutingUserAction extends Struct {
  constructor(value?: any) {
    super(registry, { destination: "VersionedMultiLocation" }, value);
  }
}
class XcmRoutingUserActionWithFee extends Struct {
  constructor(value?: any) {
    super(registry, { destination: "VersionedMultiLocation", fee: "U256" }, value);
  }
}

describeDevMoonbeam(`Test GMP Killswitch`, (context) => {
  it("should fail with killswitch enabled by default", async function () {
    // payload should be irrelevant since the precompile will fail before attempting to decode
    const transferVAA = "deadbeef";

    const data = GMP_INTERFACE.encodeFunctionData("wormholeTransferERC20", [`0x${transferVAA}`]);

    const result = await context.createBlock(
      createTransaction(context, {
        to: PRECOMPILE_GMP_ADDRESS,
        gas: 500_000,
        data,
      })
    );

    expectEVMResult(result.result.events, "Revert", "Reverted");
    const revertReason = await extractRevertReason(result.result.hash, context.ethers);
    expect(revertReason).to.contain("GMP Precompile is not enabled");
  });
});
