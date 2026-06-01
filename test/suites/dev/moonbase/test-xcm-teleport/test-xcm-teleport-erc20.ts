import "@moonbeam-network/api-augment";
import {
  ALITH_ADDRESS,
  CHARLETH_ADDRESS,
  alith,
  beforeAll,
  charleth,
  describeSuite,
  expect,
} from "moonwall";
import type { ApiPromise } from "@polkadot/api";
import { u8aToHex } from "@polkadot/util";
import { expectEVMResult, sealExtrinsic, sealExtrinsics } from "../../../../helpers";
import {
  XcmFragment,
  type XcmFragmentConfig,
  injectHrmpMessage,
  mockHrmpChannelExistanceTx,
  sovereignAccountOfSibling,
} from "../../../../helpers/xcm.js";

// `T::TeleportCheckingAccount::get()` for Moonbase:
//   H160(*b"erc20-teleport-check") = 0x65726332302d74656c65706f72742d636865636b
// (see runtime/moonbase/src/xcm_config.rs::Erc20TeleportCheckingAccount).
const CHECKING_ADDRESS = "0x65726332302d74656c65706f72742d636865636b" as const;

const TOTAL_SUPPLY = 1_000_000n;
const SEED_AMOUNT = 1_000n;
const DRAIN_AMOUNT = 400n;

// Moonbase `TeleportTrustedLocation` / `AssetHubLocation` (runtime/moonbase/src/xcm_config.rs).
const MOONBASE_ASSET_HUB_PARA_ID = 1001;
const TELEPORT_OUT_AMOUNT = 500n;
const TELEPORT_IN_AMOUNT = 200n;
const INBOUND_XCM_FEE = 10n ** 17n;

function erc20Multilocation(palletIndex: number, contract: `0x${string}`) {
  return {
    parents: 0,
    interior: {
      X2: [{ PalletInstance: palletIndex }, { AccountKey20: { network: null, key: contract } }],
    },
  };
}

function assetHubVersionedLocation() {
  return {
    V5: {
      parents: 1,
      interior: { X1: [{ Parachain: MOONBASE_ASSET_HUB_PARA_ID }] },
    },
  };
}

function mockAssetHubHrmpChannelTx(
  context: Parameters<typeof sealExtrinsic>[0],
  polkadotJs: ApiPromise
) {
  return polkadotJs.tx.sudo.sudo(
    mockHrmpChannelExistanceTx(context, MOONBASE_ASSET_HUB_PARA_ID, 1000, 102400, 102400)
  );
}

describeSuite({
  id: "D024501",
  title:
    "XCM teleport ERC-20 — mock Asset Hub HRMP happy path and deregistered " +
    "split-transactor regression",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let polkadotJs: ApiPromise;
    let teleportErc20ContractAddress: `0x${string}`;
    let erc20PalletIndex: number;
    let balancesPalletIndex: number;

    beforeAll(async () => {
      polkadotJs = context.polkadotJs();

      const metadata = await polkadotJs.rpc.state.getMetadata();
      erc20PalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Erc20XcmBridge")!
        .index.toNumber();
      balancesPalletIndex = metadata.asLatest.pallets
        .find(({ name }) => name.toString() === "Balances")!
        .index.toNumber();

      expect(
        (polkadotJs.tx as any).erc20XcmBridge?.addTeleportableErc20,
        "runtime does not expose `erc20XcmBridge.addTeleportableErc20`"
      ).toBeDefined();
    });

    it({
      id: "T01",
      title:
        "Alith teleports TELEPORT_OUT_AMOUNT to mock Asset Hub on a fresh whitelisted ERC-20 " +
        "(locks supply in TeleportCheckingAccount, state = Active)",
      test: async () => {
        const { contractAddress, status } = await context.deployContract!(
          "ERC20WithInitialSupply",
          {
            args: ["TeleportCoin", "TPC", ALITH_ADDRESS, TOTAL_SUPPLY],
          }
        );
        teleportErc20ContractAddress = contractAddress as `0x${string}`;
        expect(status).to.equal("success");

        const erc20Location = erc20Multilocation(erc20PalletIndex, teleportErc20ContractAddress);
        const beneficiaryOnAh = {
          V5: {
            parents: 0,
            interior: {
              X1: [{ AccountKey20: { network: null, key: ALITH_ADDRESS } }],
            },
          },
        };
        const versionedAssets = polkadotJs.createType("XcmVersionedAssets", {
          V5: [
            {
              id: erc20Location,
              fun: { Fungible: TELEPORT_OUT_AMOUNT },
            },
          ],
        });

        await sealExtrinsics(
          context,
          [
            mockAssetHubHrmpChannelTx(context, polkadotJs),
            polkadotJs.tx.sudo.sudo(
              (polkadotJs.tx as any).erc20XcmBridge.addTeleportableErc20(
                teleportErc20ContractAddress
              )
            ),
            polkadotJs.tx.polkadotXcm.limitedTeleportAssets(
              assetHubVersionedLocation(),
              beneficiaryOnAh,
              versionedAssets,
              0,
              { Unlimited: null }
            ),
          ],
          alith
        );

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: teleportErc20ContractAddress,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).to.equal(TOTAL_SUPPLY - TELEPORT_OUT_AMOUNT);
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: teleportErc20ContractAddress,
            functionName: "balanceOf",
            args: [CHECKING_ADDRESS],
          })
        ).to.equal(TELEPORT_OUT_AMOUNT);
        expect(
          (
            await (polkadotJs.query as any).erc20XcmBridge.lockedSupply(
              teleportErc20ContractAddress
            )
          ).toBigInt()
        ).to.equal(TELEPORT_OUT_AMOUNT);

        const whitelistStatus = await (polkadotJs.query as any).erc20XcmBridge.teleportableErc20s(
          teleportErc20ContractAddress
        );
        expect(whitelistStatus.toString()).to.equal("Active");
      },
    });

    it({
      id: "T02",
      title:
        "Mock Asset Hub delivers ReceiveTeleportedAsset + DepositAsset for the teleport ERC-20 " +
        "(unwinds locked supply to Charleth)",
      test: async () => {
        const selfReserve = {
          parents: 0,
          interior: { X1: { PalletInstance: balancesPalletIndex } },
        };
        const erc20Location = erc20Multilocation(erc20PalletIndex, teleportErc20ContractAddress);

        // Fund the AH sibling sovereign so the inbound XCM can withdraw DEV for
        // `BuyExecution` (Moonbase's barrier only admits top-level paid execution).
        const assetHubSovereign = sovereignAccountOfSibling(context, MOONBASE_ASSET_HUB_PARA_ID);
        await sealExtrinsic(
          context,
          polkadotJs.tx.balances.transferAllowDeath(assetHubSovereign, INBOUND_XCM_FEE * 2n),
          alith
        );

        // Withdraw native fees from the AH sovereign first (`AllowTopLevelPaidExecutionFrom`
        // wants `BuyExecution` right after `WithdrawAsset` / `ReceiveTeleportedAsset`), then
        // credit the teleported ERC-20 and deposit it to Charleth from the checking account.
        const inboundXcm = new XcmFragment({
          assets: [{ multilocation: selfReserve, fungible: INBOUND_XCM_FEE }],
          weight_limit: {
            refTime: 40_000_000_000n,
            proofSize: 200_000n,
          },
          beneficiary: CHARLETH_ADDRESS,
        })
          .withdraw_asset()
          .buy_execution(0)
          .push_any({
            ReceiveTeleportedAsset: [
              {
                id: erc20Location,
                fun: { Fungible: TELEPORT_IN_AMOUNT },
              },
            ],
          })
          .deposit_asset_definite(erc20Location, TELEPORT_IN_AMOUNT, CHARLETH_ADDRESS)
          .as_v5();

        await mockAssetHubHrmpChannelTx(context, polkadotJs).signAndSend(alith);
        await injectHrmpMessage(context, MOONBASE_ASSET_HUB_PARA_ID, {
          type: "XcmVersionedXcm",
          payload: inboundXcm,
        });
        await context.createBlock();

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: teleportErc20ContractAddress,
            functionName: "balanceOf",
            args: [CHARLETH_ADDRESS],
          })
        ).to.equal(TELEPORT_IN_AMOUNT);
        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: teleportErc20ContractAddress,
            functionName: "balanceOf",
            args: [CHECKING_ADDRESS],
          })
        ).to.equal(TELEPORT_OUT_AMOUNT - TELEPORT_IN_AMOUNT);
        expect(
          (
            await (polkadotJs.query as any).erc20XcmBridge.lockedSupply(
              teleportErc20ContractAddress
            )
          ).toBigInt()
        ).to.equal(TELEPORT_OUT_AMOUNT - TELEPORT_IN_AMOUNT);

        const whitelistStatus = await (polkadotJs.query as any).erc20XcmBridge.teleportableErc20s(
          teleportErc20ContractAddress
        );
        expect(whitelistStatus.toString()).to.equal("Active");
      },
    });

    it({
      id: "T03",
      title:
        "Charleth cannot drain checking via polkadotXcm.execute " +
        "([WithdrawAsset, BuyExecution, DepositAsset])",
      test: async () => {
        const { contractAddress, status: deployStatus } = await context.deployContract!(
          "ERC20WithInitialSupply",
          {
            args: ["DrainCoin", "DRC", ALITH_ADDRESS, TOTAL_SUPPLY],
          }
        );
        const erc20ContractAddress = contractAddress as `0x${string}`;
        expect(deployStatus).to.equal("success");
        log(`Deployed ERC-20 at ${erc20ContractAddress}`);

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).to.equal(TOTAL_SUPPLY);

        // Sudo whitelists the ERC-20 (state = Registered)
        await sealExtrinsic(
          context,
          polkadotJs.tx.sudo.sudo(
            (polkadotJs.tx as any).erc20XcmBridge.addTeleportableErc20(erc20ContractAddress)
          ),
          alith
        );

        const registeredStatus = await (polkadotJs.query as any).erc20XcmBridge.teleportableErc20s(
          erc20ContractAddress
        );
        expect(registeredStatus.toString()).to.equal("Registered");

        // Alith transfers SEED_AMOUNT directly to TeleportCheckingAccount
        const rawTx = await context.writeContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress,
          functionName: "transfer",
          args: [CHECKING_ADDRESS, SEED_AMOUNT],
          rawTxOnly: true,
        });
        const { result: transferResult } = await context.createBlock(rawTx);
        expectEVMResult(transferResult!.events, "Succeed");

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress,
            functionName: "balanceOf",
            args: [CHECKING_ADDRESS],
          })
        ).to.equal(SEED_AMOUNT);

        // Sudo bumps LockedSupply and then deregisters (state = Deregistered)
        const storageKey = (polkadotJs.query as any).erc20XcmBridge.lockedSupply.key(
          erc20ContractAddress
        );
        const storageValue = u8aToHex(polkadotJs.registry.createType("U256", SEED_AMOUNT).toU8a());

        await sealExtrinsic(
          context,
          polkadotJs.tx.sudo.sudo(polkadotJs.tx.system.setStorage([[storageKey, storageValue]])),
          alith
        );
        expect(
          (
            await (polkadotJs.query as any).erc20XcmBridge.lockedSupply(erc20ContractAddress)
          ).toBigInt()
        ).to.equal(SEED_AMOUNT);

        await sealExtrinsic(
          context,
          polkadotJs.tx.sudo.sudo(
            (polkadotJs.tx as any).erc20XcmBridge.removeTeleportableErc20(erc20ContractAddress)
          ),
          alith
        );
        const deregisteredStatus = await (
          polkadotJs.query as any
        ).erc20XcmBridge.teleportableErc20s(erc20ContractAddress);
        expect(deregisteredStatus.toString()).to.equal("Deregistered");
        const charlethBefore = await context.readContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress,
          functionName: "balanceOf",
          args: [CHARLETH_ADDRESS],
        });
        const checkingBefore = await context.readContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress,
          functionName: "balanceOf",
          args: [CHECKING_ADDRESS],
        });
        const lockedBefore = (
          await (polkadotJs.query as any).erc20XcmBridge.lockedSupply(erc20ContractAddress)
        ).toBigInt();

        expect(charlethBefore).to.equal(0n);
        expect(checkingBefore).to.equal(SEED_AMOUNT);
        expect(lockedBefore).to.equal(SEED_AMOUNT);

        const selfReserve = {
          parents: 0,
          interior: { X1: { PalletInstance: balancesPalletIndex } },
        };
        const erc20Location = erc20Multilocation(erc20PalletIndex, erc20ContractAddress);

        const config = {
          assets: [
            { multilocation: selfReserve, fungible: 10n ** 18n },
            { multilocation: erc20Location, fungible: DRAIN_AMOUNT },
          ],
          beneficiary: CHARLETH_ADDRESS,
        } as XcmFragmentConfig;

        const xcmMessage = new XcmFragment(config)
          .withdraw_asset()
          .buy_execution(0)
          .deposit_asset(2n)
          .as_v5();

        const { result: executeResult } = await sealExtrinsic(
          context,
          polkadotJs.tx.polkadotXcm.execute(xcmMessage, {
            refTime: 20_000_000_000n,
            proofSize: 1_000_000n,
          }),
          charleth,
          { createBlock: { allowFailures: true } }
        );

        expect(executeResult!.successful, "attack XCM must not complete successfully").to.be.false;

        const charlethAfter = await context.readContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress,
          functionName: "balanceOf",
          args: [CHARLETH_ADDRESS],
        });
        const checkingAfter = await context.readContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress,
          functionName: "balanceOf",
          args: [CHECKING_ADDRESS],
        });
        const lockedAfter = (
          await (polkadotJs.query as any).erc20XcmBridge.lockedSupply(erc20ContractAddress)
        ).toBigInt();

        log(`Charleth ERC-20: ${charlethBefore} -> ${charlethAfter}`);
        log(`Checking ERC-20: ${checkingBefore} -> ${checkingAfter}`);
        log(`LockedSupply:    ${lockedBefore} -> ${lockedAfter}`);

        expect(charlethAfter).to.equal(0n);
        expect(checkingAfter).to.equal(SEED_AMOUNT);
        expect(lockedAfter).to.equal(SEED_AMOUNT);
      },
    });
  },
});
