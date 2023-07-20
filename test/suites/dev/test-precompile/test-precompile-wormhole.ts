import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  ALITH_PRIVATE_KEY,
  PRECOMPILES,
  alith,
  createViemTransaction,
} from "@moonwall/util";
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
  EvmChainId: 1287
  Finality: 1
  Implementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 
  Wormhole: 0xa5b7d85a8f27dd7907dc8fdc21fa5657d5e2f901
  TokenImplementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 

  TokenImplementation: 0x7d9a2fc0d5d0d12b0f943930a4ba1a1233637fc9 
*/

const GUARDIAN_SET_INDEX = 0;

describeSuite({
  id: "D2568",
  title: "Test local Wormhole",
  foundationMethods: "dev",

  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should support Alith VAA",
      timeout: 3600 * 1000,
      test: async function () {
        const { contractAddress: weth9Addr } = await context.deployContract!("MockWETH9");
        log(`WETH contract deployed to ${weth9Addr}`);

        const initialSigners = [ALITH_ADDRESS];
        const signerPKs = [ALITH_PRIVATE_KEY];
        const chainId = "0x10";
        const governanceChainId = "0x1";
        const governanceContract =
          "0x0000000000000000000000000000000000000000000000000000000000000004";
        const evmChainId = await context.viem().getChainId(); //"1337"; // "1281";
        // Deploy wormhole (based on wormhole)
        // wormhole-foundation/wormhole/blob/main/ethereum/migrations/2_deploy_wormhole.js
        const { contractAddress: setupAddr, abi: setupAbi } = await context.deployContract!(
          "Setup"
        );
        const { contractAddress: implAddr } = await context.deployContract!("Implementation");

        const wormholeSetupData = encodeFunctionData({
          abi: setupAbi,
          functionName: "setup",
          args: [
            implAddr,
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
        const { contractAddress: bridgeImplAddr, abi: bridgeImplAbi } =
          await context.deployContract!("BridgeImplementation");

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
            weth9Addr,
            finality,
            evmChainId,
          ],
        });

        const { contractAddress: bridgeAddr } = await context.deployContract!("TokenBridge", {
          args: [bridgeSetupAddr, bridgeSetupData],
        });
        log(`bridge contract deployed to ${bridgeAddr}`);

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

        const registerTxn = await createViemTransaction(context, {
          to: bridgeAddr,
          data: encodeFunctionData({
            abi: bridgeImplAbi,
            functionName: "registerChain",
            args: [`0x${registerChainVm}`],
          }),
        });

        const registerChainResult = await context.createBlock(registerTxn);
        log(
          `Registered chain ${ETHChain}: ${ETHEmitter} => ${registerChainResult.result!.hash} (${
            registerChainResult.result!.successful || "good"
          })`
        );

        // Register Asset MyToken
        const assetMetaVm = await genAssetMeta(
          signerPKs,
          GUARDIAN_SET_INDEX,
          nonce++,
          1,
          weth9Addr,
          ETHChain,
          ETHEmitter,
          18,
          "WETH",
          "Wrapped Ether"
        );

        const assetMetaTxn = await createViemTransaction(context, {
          to: bridgeAddr,
          data: encodeFunctionData({
            abi: bridgeImplAbi,
            functionName: "createWrapped",
            args: [`0x${assetMetaVm}`],
          }),
        });
        const assetMetaResult = await context.createBlock(assetMetaTxn);

        const wrappedToken = (
          await context
            .viem()
            .getTransactionReceipt({ hash: assetMetaResult!.result!.hash as `0x${string}` })
        ).logs[0].address;

        log(
          `Created Wrapped Asset ${wrappedToken} => ${assetMetaResult.result!.hash} (${
            assetMetaResult.result!.error || "good"
          })`
        );

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
            context
              .polkadotJs()
              .tx.system.setStorage([[CORE_CONTRACT_STORAGE_ADDRESS, wormholeAddr]])
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
            context
              .polkadotJs()
              .tx.system.setStorage([[BRIDGE_CONTRACT_STORAGE_ADDRESS, bridgeAddr]])
          )
          .signAndSend(alith);
        await context.createBlock();

        // create payload
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

        const destination = context
          .polkadotJs()
          .registry.createType("VersionedMultiLocation", versionedMultiLocation);

        // we also need to disable the killswitch by setting the 'enabled' flag to Some(true)
        const ENABLED_FLAG_STORAGE_ADDRESS = u8aToHex(
          u8aConcat(xxhashAsU8a("gmp", 128), xxhashAsU8a("PrecompileEnabled", 128))
        );
        expect(ENABLED_FLAG_STORAGE_ADDRESS).to.eq(
          "0xb7f047395bba5df0367b45771c00de502551bba17abb82ef3498bab688e470b8"
        );

        const userAction = new XcmRoutingUserAction({ destination });
        const versionedUserAction = new VersionedUserAction({ V1: userAction });
        console.log("Versioned User Action JSON:", JSON.stringify(versionedUserAction.toJSON()));
        console.log("Versioned User Action SCALE:", versionedUserAction.toHex());
        let payload = "" + versionedUserAction.toHex();

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

        const transferVAA = await genTransferWithPayloadVAA(
          signerPKs,
          GUARDIAN_SET_INDEX,
          nonce++,
          123, // sequence
          999, // amount of tokens
          weth9Addr,
          ETHChain,
          ETHChain,
          ETHEmitter, // TODO: review
          PRECOMPILES.Gmp,
          "0x" + evmChainId.toString(16),
          "0x0000000000000000000000000000000000000001", // TODO: fromAddress
          "" + payload
        );

        const transferErc20Txn = await context.writePrecompile!({
          precompileName: "Gmp",
          functionName: "wormholeTransferERC20",
          args: [`0x${transferVAA}`],
          rawTxOnly: true,
        });

        const result = await context.createBlock(transferErc20Txn);

        expectEVMResult(result.result!.events, "Succeed", "Returned");
        expectSubstrateEvents(result, "xTokens", "TransferredMultiAssets");
      },
    });
  },
});

const registry = new TypeRegistry();

class VersionedUserAction extends Enum {
  constructor(value?: any) {
    super(registry, { V1: XcmRoutingUserAction }, value);
  }
}
class XcmRoutingUserAction extends Struct {
  constructor(value?: any) {
    super(registry, { destination: "VersionedMultiLocation" }, value);
  }
}
