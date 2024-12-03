import { describeSuite, expect, beforeAll } from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { ethers, parseEther } from "ethers";
import { expectOk } from "../../../../helpers";
import {
  registerOldForeignAsset,
  PARA_1000_SOURCE_LOCATION,
  assetContractAddress,
  mockOldAssetBalance,
  foreignAssetBalance,
} from "../../../../helpers/assets";
import {
  ALITH_ADDRESS,
  BALTATHAR_ADDRESS,
  CHARLETH_ADDRESS,
  alith,
  createEthersTransaction,
} from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";
import { DispatchResult } from "@polkadot/types/interfaces/system/types";
import { encodeFunctionData, parseAbi } from "viem";

describeSuite({
  id: "D012202",
  title: "Lazy Migrations Pallet - Foreign Asset Migration",
  foundationMethods: "dev",
  testCases: ({ context, it }) => {
    let api: ApiPromise;
    let assetId: u128;

    beforeAll(async () => {
      api = context.polkadotJs();

      // Register foreign asset using helper
      const { registeredAssetId } = await registerOldForeignAsset(
        context,
        PARA_1000_SOURCE_LOCATION,
        {
          name: "Foreign Asset",
          symbol: "FA",
          decimals: 18n,
          isFrozen: false,
        }
      );

      assetId = context.polkadotJs().createType("u128", registeredAssetId);

      // Define test accounts and their balances
      const accounts = [
        { address: ALITH_ADDRESS, balance: "100" },
        { address: BALTATHAR_ADDRESS, balance: "50" },
        { address: CHARLETH_ADDRESS, balance: "25" },
      ];

      const totalSupply = accounts
        .reduce((sum, account) => sum + parseFloat(account.balance), 0)
        .toString();

      // Create asset details
      const assetDetails = context.polkadotJs().createType("PalletAssetsAssetDetails", {
        supply: parseEther(totalSupply),
        owner: ALITH_ADDRESS,
        deposit: 1,
        isSufficient: false,
        minBalance: 1,
        isFrozen: false,
        sufficients: 0,
        approvals: 0,
      });

      // Create balances for all test accounts
      for (const { address, balance } of accounts) {
        const assetBalance = context.polkadotJs().createType("PalletAssetsAssetAccount", {
          balance: parseEther(balance),
          isFrozen: false,
          reason: "Consumer",
          extra: null,
        });

        await mockOldAssetBalance(context, assetBalance, assetDetails, alith, assetId, address);
      }
      await context.createBlock([]);
      // Create approvals
      await expectOk(
        context.createBlock(
          api.tx.assets.approveTransfer(assetId, BALTATHAR_ADDRESS, parseEther("10")),
          { signer: alith, allowFailures: false }
        )
      );

      await expectOk(
        context.createBlock(
          api.tx.assets.approveTransfer(assetId, CHARLETH_ADDRESS, parseEther("5")),
          { signer: alith, allowFailures: false }
        )
      );
    });

    it({
      id: "T01",
      title: "Should not allow non-root to start migration",
      test: async function () {
        const { result } = await context.createBlock(
          api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId)
        );

        expect(result?.error?.name).to.equal("BadOrigin");
      },
    });

    it({
      id: "T02",
      title: "Should start migration and freeze asset",
      test: async function () {
        // Start migration with sudo
        await expectOk(
          context.createBlock(
            api.tx.sudo.sudo(api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId))
          )
        );

        // Asset should be frozen
        const assetDetails = await api.query.assets.asset(assetId);
        expect(assetDetails.unwrap().status.isFrozen).to.be.true;

        // Transfers should fail
        const { result } = await context.createBlock(
          api.tx.assets.transfer(assetId, BALTATHAR_ADDRESS, parseEther("10"))
        );
        expect(result?.error?.name).to.equal("AssetNotLive");

        // Attempt to start another migration
        const { result: res } = await context.createBlock(
          api.tx.sudo.sudo(
            api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId.toBigInt() + 1n)
          )
        );

        // Find Sudid event and extract inner result
        const sudidEvent = res?.events.find((e) => api.events.sudo.Sudid.is(e.event));
        const innerResult = sudidEvent?.event.data[0] as DispatchResult;

        // Verify inner transaction failed with correct error
        const { section, name } = api.registry.findMetaError(innerResult.asErr.asModule);
        expect(`${section}.${name}`).to.equal("moonbeamLazyMigrations.MigrationNotFinished");
      },
    });

    it({
      id: "T03",
      title: "Should handle migrating multiple balances and approvals with proper cleanup",
      test: async function () {
        const accounts = [ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS];

        // Verify initial balances
        const initialBalances = await Promise.all([
          foreignAssetBalance(context, assetId.toBigInt(), ALITH_ADDRESS),
          foreignAssetBalance(context, assetId.toBigInt(), BALTATHAR_ADDRESS),
          foreignAssetBalance(context, assetId.toBigInt(), CHARLETH_ADDRESS),
        ]);

        // Verify initial approvals
        const initialApprovals = await Promise.all([
          api.query.assets.approvals(assetId, ALITH_ADDRESS, BALTATHAR_ADDRESS),
          api.query.assets.approvals(assetId, ALITH_ADDRESS, CHARLETH_ADDRESS),
        ]);

        // Start migration
        const alithBalanceBefore = await api.query.system.account(ALITH_ADDRESS);
        await expectOk(
          context.createBlock(
            api.tx.sudo.sudo(api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId))
          )
        );

        // 2. Execute migration
        await expectOk(
          context.createBlock(api.tx.moonbeamLazyMigrations.migrateForeignAssetBalances(3))
        );

        // Check that migration is not finished
        const { result } = await context.createBlock(
          api.tx.moonbeamLazyMigrations.finishForeignAssetsMigration()
        );
        expect(result?.error?.name).to.equal("MigrationNotFinished");

        // Migrate approvals
        await expectOk(
          context.createBlock(api.tx.moonbeamLazyMigrations.migrateForeignAssetApprovals(3))
        );

        await expectOk(
          context.createBlock(api.tx.moonbeamLazyMigrations.finishForeignAssetsMigration())
        );

        // 3. Verify migration success
        expect((await api.query.assets.asset(assetId)).isNone).to.be.true;

        // Verify reserved was unreserved
        const alithAccountBalanceAfter = await api.query.system.account(ALITH_ADDRESS);
        expect(alithAccountBalanceAfter.data.reserved.toBigInt()).to.equal(
          alithBalanceBefore.data.reserved.toBigInt() - 1n
        );

        // Verify new asset functionality
        const erc20Abi = [
          "function decimals() external view returns (uint8)",
          "function symbol() external view returns (string)",
          "function name() external view returns (string)",
          "function balanceOf(address) external view returns (uint256)",
          "function allowance(address owner, address spender) external view returns (uint256)",
        ];

        // Get contract address from assetId
        const contractAddress = assetContractAddress(assetId.toBigInt());
        const foreignAssetContract = new ethers.Contract(
          contractAddress,
          erc20Abi,
          context.ethers()
        );

        // Query decimals and verify
        const decimals = await foreignAssetContract.decimals();
        expect(decimals).to.equal(18n);

        // Check balances were properly migrated
        await Promise.all(
          accounts.map((account, index) =>
            foreignAssetContract
              .balanceOf(account)
              .then((balance) => expect(balance).to.equal(initialBalances[index]))
          )
        );

        // Verify approvals were migrated correctly
        const migratedAllowances = await Promise.all([
          foreignAssetContract.allowance(ALITH_ADDRESS, BALTATHAR_ADDRESS),
          foreignAssetContract.allowance(ALITH_ADDRESS, CHARLETH_ADDRESS),
        ]);
        expect(migratedAllowances[0].toString()).to.equal(
          initialApprovals[0].unwrap().amount.toString()
        );
        expect(migratedAllowances[1].toString()).to.equal(
          initialApprovals[1].unwrap().amount.toString()
        );

        // Test transfer
        const transferAmount = parseEther("5");
        const [alithBefore, baltatharBefore] = await Promise.all([
          foreignAssetContract.balanceOf(ALITH_ADDRESS),
          foreignAssetContract.balanceOf(BALTATHAR_ADDRESS),
        ]);

        await context.createBlock(
          await createEthersTransaction(context, {
            to: contractAddress,
            data: encodeFunctionData({
              abi: parseAbi(["function transfer(address,uint256)"]),
              functionName: "transfer",
              args: [BALTATHAR_ADDRESS, transferAmount],
            }),
          })
        );

        expect(await foreignAssetBalance(context, assetId.toBigInt(), ALITH_ADDRESS)).to.equal(
          alithBefore - transferAmount
        );
        expect(await foreignAssetBalance(context, assetId.toBigInt(), BALTATHAR_ADDRESS)).to.equal(
          baltatharBefore + transferAmount
        );

        // Test approve & transferFrom
        const approvalAmount = parseEther("20");
        await context.createBlock(
          await createEthersTransaction(context, {
            to: contractAddress,
            data: encodeFunctionData({
              abi: parseAbi(["function approve(address,uint256)"]),
              functionName: "approve",
              args: [CHARLETH_ADDRESS, approvalAmount],
            }),
          })
        );

        // Check approval
        expect(await foreignAssetContract.allowance(ALITH_ADDRESS, CHARLETH_ADDRESS)).to.equal(
          approvalAmount
        );
      },
    });
  },
});
