import {
    describeSuite,
    expect,
    beforeAll,
} from "@moonwall/cli";
import { ApiPromise } from "@polkadot/api";
import { ethers, parseEther } from "ethers";
import { expectOk } from "../../../../helpers";
import {
    registerOldForeignAsset,
    PARA_1000_SOURCE_LOCATION_V3,
    assetContractAddress,
    mockOldAssetBalance,
} from "../../../../helpers/assets";
import { ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS, alith } from "@moonwall/util";
import { u128 } from "@polkadot/types-codec";

describeSuite({
    id: "D012202",
    title: "Lazy Migrations Pallet - Foreign Asset Migration",
    foundationMethods: "dev",
    testCases: ({ context, it, log }) => {
        let api: ApiPromise;
        let assetId: u128;

        beforeAll(async () => {
            api = context.polkadotJs();

            // Register foreign asset using helper
            const { registeredAssetId } = await registerOldForeignAsset(
                context,
                PARA_1000_SOURCE_LOCATION_V3,
                {
                    name: "Foreign Asset",
                    symbol: "FA",
                    decimals: 18n,
                    isFrozen: false,
                }
            );

            assetId = context.polkadotJs().createType("u128", registeredAssetId);
            log(`Created foreign asset with ID: ${assetId}`);

            // Define test accounts and their balances
            const accounts = [
                { address: ALITH_ADDRESS, balance: "100" },
                { address: BALTATHAR_ADDRESS, balance: "50" },
                { address: CHARLETH_ADDRESS, balance: "25" }
            ];

            const totalSupply = accounts.reduce(
                (sum, account) => sum + parseFloat(account.balance), 0
            ).toString();

            // Create asset details
            const assetDetails = context.polkadotJs().createType("PalletAssetsAssetDetails", {
                supply: parseEther(totalSupply),
                owner: ALITH_ADDRESS,
                deposit: 1,
                isSufficient: false,
                minBalance: 1,
                isFrozen: false,
                sufficients: 0,
                approvals: 0
            });

            // Create balances for all test accounts
            for (const { address, balance } of accounts) {
                const assetBalance = context.polkadotJs().createType("PalletAssetsAssetAccount", {
                    balance: parseEther(balance),
                    isFrozen: false,
                    reason: "Consumer",
                    extra: null
                });

                await mockOldAssetBalance(
                    context,
                    assetBalance,
                    assetDetails,
                    alith,
                    assetId,
                    address
                );
            }

            // Create approvals
            await expectOk(
                context.createBlock([
                    api.tx.assets.approveTransfer(assetId, BALTATHAR_ADDRESS, parseEther("10")),
                    api.tx.assets.approveTransfer(assetId, CHARLETH_ADDRESS, parseEther("5")),
                ])
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
                        api.tx.sudo.sudo(
                            api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId)
                        )
                    )
                );

                // Asset should be frozen
                const assetDetails = await api.query.assets.asset(assetId);
                expect(assetDetails.unwrap().status.isFrozen).to.be.true;

                // Transfers should fail
                const { result } = await context.createBlock(
                    api.tx.assets.transfer(assetId, BALTATHAR_ADDRESS, parseEther("10"))
                );
                expect(result?.error?.name).to.equal("Frozen");

                // Attempt to start another migration
                const { result: res } = await context.createBlock(
                    api.tx.sudo.sudo(
                        api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId.toBigInt() + 1n)
                    )
                );

                expect(res?.error?.name).to.equal("MigrationNotFinished");
            },
        });

        it({
            id: "T03",
            title: "Should handle migrating multiple balances and approvals with propper cleanup",
            test: async function () {
                const accounts = [ALITH_ADDRESS, BALTATHAR_ADDRESS, CHARLETH_ADDRESS];
                const balances = ["100", "50", "25"];

                // Verify initial approvals
                const initialApprovals = await Promise.all([
                    api.query.assets.approvals(assetId, ALITH_ADDRESS, BALTATHAR_ADDRESS),
                    api.query.assets.approvals(assetId, ALITH_ADDRESS, CHARLETH_ADDRESS),
                ]);
                expect(initialApprovals[0].unwrap().amount.toString()).to.equal(parseEther("30").toString());
                expect(initialApprovals[1].unwrap().amount.toString()).to.equal(parseEther("20").toString());


                const reservedBalances = await Promise.all(
                    accounts.map(async (acc) => {
                        const accountInfo = await api.query.system.account(acc);
                        return accountInfo.data.reserved;
                    })
                );

                // Verify reserved balances
                accounts.forEach((_, index) => {
                    expect(reservedBalances[index].toString())
                        .to.equal(parseEther(balances[index]).toString());
                });

                // Start migration
                await expectOk(
                    context.createBlock(
                        api.tx.sudo.sudo(
                            api.tx.moonbeamLazyMigrations.startForeignAssetsMigration(assetId)
                        )
                    )
                );

                // Migrate remaining balances
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

                // Verify reserves were unreserved
                const finalReserves = await Promise.all(
                    accounts.map(acc => api.query.system.account(acc))
                );

                // Compare reserves
                accounts.forEach((_, index) => {
                    expect(finalReserves[index].data.reserved.toString())
                        .to.equal(0);
                });

                // Complete migration
                await expectOk(
                    context.createBlock(api.tx.moonbeamLazyMigrations.finishForeignAssetsMigration())
                );

                // Verify cleanup
                const oldAsset = await api.query.assets.asset(assetId);
                expect(oldAsset.isNone).to.be.true;

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
                expect(decimals).to.equal(18);

                // Additional verification - check balances were properly migrated
                const contractBalance = await foreignAssetContract.balanceOf(ALITH_ADDRESS);
                expect(contractBalance.toString()).to.equal(parseEther(balances[0]).toString());


                // Verify approvals were migrated correctly
                const migratedAllowances = await Promise.all([
                    foreignAssetContract.allowance(ALITH_ADDRESS, BALTATHAR_ADDRESS),
                    foreignAssetContract.allowance(ALITH_ADDRESS, CHARLETH_ADDRESS),
                ]);

                expect(migratedAllowances[0].toString()).to.equal(parseEther("30").toString());
                expect(migratedAllowances[1].toString()).to.equal(parseEther("20").toString());

            },
        });
    },
});
