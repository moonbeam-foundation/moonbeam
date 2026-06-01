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
import { expectEVMResult, sealExtrinsic } from "../../../../helpers";
import { XcmFragment, type XcmFragmentConfig } from "../../../../helpers/xcm.js";

// `T::TeleportCheckingAccount::get()` for Moonbase:
//   H160(*b"erc20-teleport-check") = 0x65726332302d74656c65706f72742d636865636b
// (see runtime/moonbase/src/xcm_config.rs::Erc20TeleportCheckingAccount).
const CHECKING_ADDRESS = "0x65726332302d74656c65706f72742d636865636b" as const;

const TOTAL_SUPPLY = 1_000_000n;
const SEED_AMOUNT = 1_000n;
const DRAIN_AMOUNT = 400n;

describeSuite({
  id: "D024501",
  title:
    "XCM teleport ERC-20 — Deregistered split-transactor regression " +
    "(polkadotXcm.execute cannot drain TeleportCheckingAccount)",
  foundationMethods: "dev",
  testCases: ({ it, context, log }) => {
    let polkadotJs: ApiPromise;
    let erc20ContractAddress: `0x${string}`;
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
      title: "Alith deploys a real ERC-20 (full supply to Alith)",
      test: async () => {
        const { contractAddress, status } = await context.deployContract!(
          "ERC20WithInitialSupply",
          {
            args: ["DrainCoin", "DRC", ALITH_ADDRESS, TOTAL_SUPPLY],
          }
        );
        erc20ContractAddress = contractAddress as `0x${string}`;
        expect(status).to.equal("success");
        log(`Deployed ERC-20 at ${erc20ContractAddress}`);

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress,
            functionName: "balanceOf",
            args: [ALITH_ADDRESS],
          })
        ).to.equal(TOTAL_SUPPLY);
      },
    });

    it({
      id: "T02",
      title: "Sudo whitelists the ERC-20 (state = Registered)",
      test: async () => {
        await sealExtrinsic(
          context,
          polkadotJs.tx.sudo.sudo(
            (polkadotJs.tx as any).erc20XcmBridge.addTeleportableErc20(erc20ContractAddress)
          ),
          alith
        );

        const status = await (polkadotJs.query as any).erc20XcmBridge.teleportableErc20s(
          erc20ContractAddress
        );
        expect(status.toString()).to.equal("Registered");
      },
    });

    it({
      id: "T03",
      title: "Alith transfers SEED_AMOUNT directly to TeleportCheckingAccount",
      test: async () => {
        const rawTx = await context.writeContract!({
          contractName: "ERC20WithInitialSupply",
          contractAddress: erc20ContractAddress,
          functionName: "transfer",
          args: [CHECKING_ADDRESS, SEED_AMOUNT],
          rawTxOnly: true,
        });
        const { result } = await context.createBlock(rawTx);
        expectEVMResult(result!.events, "Succeed");

        expect(
          await context.readContract!({
            contractName: "ERC20WithInitialSupply",
            contractAddress: erc20ContractAddress,
            functionName: "balanceOf",
            args: [CHECKING_ADDRESS],
          })
        ).to.equal(SEED_AMOUNT);
      },
    });

    it({
      id: "T04",
      title: "Sudo bumps LockedSupply and then deregisters (state = Deregistered)",
      test: async () => {
        const storageKey = (polkadotJs.query as any).erc20XcmBridge.lockedSupply.key(
          erc20ContractAddress
        );
        const storageValue = u8aToHex(
          polkadotJs.registry.createType("U256", SEED_AMOUNT).toU8a()
        );

        await sealExtrinsic(
          context,
          polkadotJs.tx.sudo.sudo(
            polkadotJs.tx.system.setStorage([[storageKey, storageValue]])
          ),
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
        const status = await (polkadotJs.query as any).erc20XcmBridge.teleportableErc20s(
          erc20ContractAddress
        );
        expect(status.toString()).to.equal("Deregistered");
      },
    });

    it({
      id: "T05",
      title:
        "Charleth cannot drain checking via polkadotXcm.execute " +
        "([WithdrawAsset, BuyExecution, DepositAsset])",
      test: async () => {
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
          interior: { X1: [{ PalletInstance: balancesPalletIndex }] },
        };
        const erc20Location = {
          parents: 0,
          interior: {
            X2: [
              { PalletInstance: erc20PalletIndex },
              { AccountKey20: { network: null, key: erc20ContractAddress } },
            ],
          },
        };

        const config: XcmFragmentConfig = {
          assets: [
            { multilocation: selfReserve, fungible: 10n ** 18n },
            { multilocation: erc20Location, fungible: DRAIN_AMOUNT },
          ],
          beneficiary: CHARLETH_ADDRESS,
        };

        const xcmMessage = new XcmFragment(config)
          .withdraw_asset()
          .buy_execution(0)
          .deposit_asset(2n)
          .as_v5();

        const { result } = await sealExtrinsic(
          context,
          polkadotJs.tx.polkadotXcm.execute(xcmMessage, {
            refTime: 20_000_000_000n,
            proofSize: 1_000_000n,
          }),
          charleth,
          { createBlock: { allowFailures: true } }
        );

        expect(result!.successful, "attack XCM must not complete successfully").to.be.false;

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
