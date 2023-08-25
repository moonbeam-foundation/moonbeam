import "@moonbeam-network/api-augment";
import { beforeAll, describeSuite, expect, fetchCompiledContract } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  PRECOMPILES,
  alith,
  createViemTransaction,
} from "@moonwall/util";
import { Contract, ethers, InterfaceAbi } from "ethers";
import { Enum, Struct, TypeRegistry } from "@polkadot/types";
import { u8aConcat, u8aToHex } from "@polkadot/util";
import { xxhashAsU8a } from "@polkadot/util-crypto";
import { encodeFunctionData } from "viem";
import { expectEVMResult } from "../../../helpers/eth-transactions.js";
import { expectSubstrateEvents } from "../../../helpers/expect.js";
import {
  genAssetMeta,
  genRegisterChainVAA,
  genTransferWithPayloadVAA,
} from "../../../helpers/wormhole.js";

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
  EvmChainId: 128
  Finality: 1
  Implementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 
  Wormhole: 0xa5b7d85a8f27dd7907dc8fdc21fa5657d5e2f901
  TokenImplementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 

  TokenImplementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 
*/

const GUARDIAN_SET_INDEX = 0;

// TODO: these constants are now stored in moonwall?
const PRECOMPILE_GMP_ADDRESS = "0x0000000000000000000000000000000000000816";

// wormhole internally "compacts" amounts. they don't use a constant (it's more complicated) but we
// can get away with a constant here.
// TODO: maybe remove the implicit behavior from the util functions in wormhole.ts?
// TODO: actually, something is wrong here -- I don't think this matches the WH compacting logic
const WH_IMPLICIT_DECIMALS = 18n;
const WH_IMPLICIT_MULTIPLIER = 10n ** WH_IMPLICIT_DECIMALS;

describeSuite({
  id: "D2568",
  title: "Test local Wormhole",
  foundationMethods: "dev",

  testCases: ({ context, it, log }) => {
    const deploy = async (contractPath: string, initData?: any[]) => {
      const contract = await context.deployContract(contractPath, {
        args: initData,
      });
      const result = await context.createBlock(contract.rawTx);
      return contract;
    };

    const makeTestVAA = async function (
      amount: number,
      action: VersionedUserAction
    ): Promise<string> {
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
    };

    const signerPKs = [ALITH_PRIVATE_KEY];
    const ETHChain = 3;
    const ETHEmitter = "0x0000000000000000000000003ee18b2214aff97000d974cf647e7c347e8fa585";

    let whNonce = 0;
    // TODO: ugh, clean this up. we don't need the WETH contract we deployed, we need the wrapped
    // version of it created by WH.
    let wethAddress: string;
    let whWethContract: ethers.Contract;

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

    beforeAll(async function () {
      const wethDeployment = await deploy("MockWETH9");
      // wethContract = wethDeployment.contract;
      wethAddress = wethDeployment.contractAddress;
      log(`weth contract deployed to ${wethAddress}`);
      const myTokenContract = await deploy("MockWETH9");

      const initialSigners = [ALITH_ADDRESS];
      const chainId = "0x10";
      const governanceChainId = "0x1";
      const governanceContract =
        "0x0000000000000000000000000000000000000000000000000000000000000004";
      evmChainId = await context.viem().getChainId();
      // Deploy wormhole (based on wormhole)
      // wormhole-foundation/wormhole/blob/main/ethereum/migrations/2_deploy_wormhole.js
      const { contractAddress: setupAddr, abi: setupAbi } = await context.deployContract!("Setup");
      const implementationContract = await deploy("Implementation");
      const wormholeSetupData = encodeFunctionData({
        abi: setupAbi,
        functionName: "setup",
        args: [
          implementationContract.contractAddress,
          initialSigners,
          evmChainId,
          governanceChainId,
          governanceContract,
          evmChainId,
        ],
      });

      const { contractAddress: wormholeAddr } = await context.deployContract!("Wormhole", {
        args: [setupAddr, wormholeSetupData],
      });

      log(`wormhole core bridge deployed to ${wormholeAddr}`);

      const finality = 1;
      // Deploy bridge (based on wormhole)
      // wormhole-foundation/wormhole/blob/main/ethereum/migrations/3_deploy_bridge.js
      const { contractAddress: tokenImplAddr } = await context.deployContract!(
        "TokenImplementation"
      );
      log(`wormhole token impl deployed to ${tokenImplAddr}`);
      const { contractAddress: bridgeSetupAddr, abi: bridgeSetupAbi } =
        await context.deployContract!("BridgeSetup");
      const { contractAddress: bridgeImplAddr, abi: bridgeImplAbi } = await context.deployContract!(
        "BridgeImplementation"
      );

      const bridgeSetupData = encodeFunctionData({
        abi: bridgeSetupAbi,
        functionName: "setup",
        args: [
          bridgeImplAddr,
          evmChainId,
          wormholeAddr,
          governanceChainId,
          governanceContract,
          tokenImplAddr,
          wethAddress,
          finality,
          evmChainId,
        ],
      });

      const { contractAddress: bridgeAddr } = await context.deployContract!("TokenBridge", {
        args: [bridgeSetupAddr, bridgeSetupData],
      });
      log(`bridge contract deployed to ${bridgeAddr}`);

      // Register Chain ETH
      const registerChainVm = await genRegisterChainVAA(
        signerPKs,
        ETHEmitter,
        GUARDIAN_SET_INDEX,
        whNonce++,
        1,
        ETHChain
      );
      let rawTx = await context.writeContract!({
        contractName: "BridgeImplementation",
        contractAddress: bridgeAddr,
        functionName: "registerChain",
        rawTxOnly: true,
        args: [`0x${registerChainVm}`],
      });
      const registerChainResult = await context.createBlock(rawTx);

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
      rawTx = await context.writeContract!({
        contractName: "BridgeImplementation",
        contractAddress: bridgeAddr,
        functionName: "createWrapped",
        rawTxOnly: true,
        args: [`0x${assetMetaVm}`],
      });
      const assetMetaResult = await context.createBlock(rawTx);
      const wrappedToken = (
        await context
          .viem()
          .getTransactionReceipt({ hash: assetMetaResult!.result!.hash as `0x${string}` })
      ).logs[0].address;
      log(
        `Created Wrapped Asset ${wrappedToken} => ${assetMetaResult.result.hash} (${
          assetMetaResult.result.error || "good"
        })`
      );

      // TODO: clean up / avoid using both web3js and ethers
      // TODO: not the right contract, but it'll probably work
      const WETH_CONTRACT_JSON = fetchCompiledContract("MockWETH9");
      const WETH_INTERFACE = WETH_CONTRACT_JSON.abi as InterfaceAbi;
      whWethContract = new ethers.Contract(wrappedToken, WETH_INTERFACE, context.ethers());

      log(`wrapped token deployed to ${wrappedToken}`);

      // before interacting with the precompile, we need to set some contract addresses from our
      // our deployments above
      const CORE_CONTRACT_STORAGE_ADDRESS = u8aToHex(
        u8aConcat(xxhashAsU8a("gmp", 128), xxhashAsU8a("CoreAddress", 128))
      );
      expect(CORE_CONTRACT_STORAGE_ADDRESS).to.eq(
        "0xb7f047395bba5df0367b45771c00de5059ff23ff65cc809711800d9d04e4b14c"
      );

      await context
        .polkadotJs()
        .tx.sudo.sudo(
          context.polkadotJs().tx.system.setStorage([[CORE_CONTRACT_STORAGE_ADDRESS, wormholeAddr]])
        )
        .signAndSend(alith);
      await context.createBlock();

      const BRIDGE_CONTRACT_STORAGE_ADDRESS = u8aToHex(
        u8aConcat(xxhashAsU8a("gmp", 128), xxhashAsU8a("BridgeAddress", 128))
      );
      expect(BRIDGE_CONTRACT_STORAGE_ADDRESS).to.eq(
        "0xb7f047395bba5df0367b45771c00de50c1586bde54b249fb7f521faf831ade45"
      );

      await context
        .polkadotJs()
        .tx.sudo.sudo(
          context.polkadotJs().tx.system.setStorage([[BRIDGE_CONTRACT_STORAGE_ADDRESS, bridgeAddr]])
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

      await context
        .polkadotJs()
        .tx.sudo.sudo(
          context
            .polkadotJs()
            .tx.system.setStorage([
              [
                ENABLED_FLAG_STORAGE_ADDRESS,
                context.polkadotJs().registry.createType("Option<bool>", true).toHex(),
              ],
            ])
        )
        .signAndSend(alith);
      await context.createBlock();
    });

    it({
      id: "T01",
      title: "should support V1 user action",
      timeout: 3600 * 1000,
      test: async function () {
        // create payload
        const destination = context
          .polkadotJs()
          .registry.createType("VersionedMultiLocation", versionedMultiLocation);

        const userAction = new XcmRoutingUserAction({ destination });
        const versionedUserAction = new VersionedUserAction({ V1: userAction });
        let payload = "" + versionedUserAction.toHex();

        const whAmount = 999n;
        const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;

        const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);

        const rawTx = await context.writePrecompile!({
          precompileName: "Gmp",
          functionName: "wormholeTransferERC20",
          args: [`0x${transferVAA}`],
          rawTxOnly: true,
        });
        const result = await context.createBlock(rawTx);

        expectEVMResult(result.result.events, "Succeed", "Returned");
        const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
        const transferFungible = events[0].data[1][0].fun;
        expect(transferFungible.isFungible);
        const transferAmount = transferFungible.asFungible.toBigInt();
        expect(transferAmount).to.eq(realAmount);
      },
    });

    it({
      id: "T02",
      title: "should support V2 user action with fee",
      timeout: 3600 * 1000,
      test: async function () {
        // create payload
        const destination = context
          .polkadotJs()
          .registry.createType("VersionedMultiLocation", versionedMultiLocation);

        const whAmount = 999n;
        const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;
        const fee = 1234500n;

        const userAction = new XcmRoutingUserActionWithFee({ destination, fee });
        const versionedUserAction = new VersionedUserAction({ V2: userAction });

        const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

        const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);

        const rawTx = await context.writePrecompile!({
          precompileName: "Gmp",
          functionName: "wormholeTransferERC20",
          args: [`0x${transferVAA}`],
          rawTxOnly: true,
        });
        const result = await context.createBlock(rawTx);

        expectEVMResult(result.result.events, "Succeed", "Returned");
        const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
        const transferFungible = events[0].data[1][0].fun;
        expect(transferFungible.isFungible);
        const transferAmount = transferFungible.asFungible.toBigInt();
        expect(transferAmount).to.eq(realAmount - fee);

        const alithWHTokenAfter = await whWethContract.balanceOf(ALITH_ADDRESS);
        expect(alithWHTokenAfter - alithWHTokenBefore).to.eq(fee);
      },
    });

    it({
      id: "T03",
      title: "should pay entire transfer when fee greater than transfer",
      timeout: 3600 * 1000,
      test: async function () {
        // create payload
        const destination = context
          .polkadotJs()
          .registry.createType("VersionedMultiLocation", versionedMultiLocation);

        const whAmount = 100n;
        const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;
        const fee = realAmount + 1n;

        const userAction = new XcmRoutingUserActionWithFee({ destination, fee });
        const versionedUserAction = new VersionedUserAction({ V2: userAction });

        const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

        const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);

        const rawTx = await context.writePrecompile!({
          precompileName: "Gmp",
          functionName: "wormholeTransferERC20",
          args: [`0x${transferVAA}`],
          rawTxOnly: true,
        });
        const result = await context.createBlock(rawTx);

        expectEVMResult(result.result.events, "Succeed", "Returned");
        // there should be no xTokens TransferredMultiAssets event since fee >= amount sent
        const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
        expect(events.length).to.eq(0); // TODO: isn't expectSubstrateEvents supposed to expect > 0?

        const alithWHTokenAfter = await whWethContract.balanceOf(ALITH_ADDRESS);
        expect(alithWHTokenAfter - alithWHTokenBefore).to.eq(realAmount);
      },
    });

    it({
      id: "T04",
      title: "should pay no fee if fee is zero",
      timeout: 3600 * 1000,
      test: async function () {
        // create payload
        const destination = context
          .polkadotJs()
          .registry.createType("VersionedMultiLocation", versionedMultiLocation);

        const whAmount = 100n;
        const realAmount = whAmount * WH_IMPLICIT_MULTIPLIER;
        const fee = 0n;

        const userAction = new XcmRoutingUserActionWithFee({ destination, fee });
        const versionedUserAction = new VersionedUserAction({ V2: userAction });

        const alithWHTokenBefore = await whWethContract.balanceOf(ALITH_ADDRESS);

        const transferVAA = await makeTestVAA(Number(whAmount), versionedUserAction);

        const rawTx = await context.writePrecompile!({
          precompileName: "Gmp",
          functionName: "wormholeTransferERC20",
          args: [`0x${transferVAA}`],
          rawTxOnly: true,
        });
        const result = await context.createBlock(rawTx);

        expectEVMResult(result.result.events, "Succeed", "Returned");
        const events = expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
        const transferFungible = events[0].data[1][0].fun;
        expect(transferFungible.isFungible);
        const transferAmount = transferFungible.asFungible.toBigInt();
        expect(transferAmount).to.eq(realAmount);

        // no fee paid
        const alithWHTokenAfter = await whWethContract.balanceOf(ALITH_ADDRESS);
        expect(alithWHTokenAfter - alithWHTokenBefore).to.eq(0n);
      },
    });
  },
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
