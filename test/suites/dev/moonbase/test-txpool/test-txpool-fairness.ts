import "@moonbeam-network/api-augment";
import { describeSuite, expect } from "@moonwall/cli";
import {
  ALITH_ADDRESS,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  ETHAN_ADDRESS,
  GLMR,
  MILLIGLMR,
  WEIGHT_PER_GAS,
  alith,
  baltathar,
  charleth,
  createRawTransfer,
  dorothy,
  ethan,
  generateKeyringPair,
  sendRawTransaction,
} from "@moonwall/util";

// for Ethereum txns, we need to send the tip as per-gas so there is no conversion necessary.
// However, we need to specify a maxFeePerGas that is high enough to allow the priority fee to
// be used as-is, e.g. it must be at least (block.baseFee + maxPriorityFeePerGas)
const HIGH_MAX_FEE_PER_GAS = GLMR;

describeSuite({
  id: "D013901",
  title: "Tip should be respected",
  foundationMethods: "dev",
  testCases: ({ context, it, log }) => {
    it({
      id: "T01",
      title: "should prefer txn with higher tip",
      test: async function () {
        const NO_TIP = 0n;
        const MED_TIP = 5n * MILLIGLMR;
        const HIGH_TIP = 20n * MILLIGLMR;

        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(dorothy.address, GLMR)
          .signAndSend(alith, { tip: NO_TIP });

        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(dorothy.address, GLMR)
          .signAndSend(baltathar, { tip: MED_TIP });

        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(dorothy.address, GLMR)
          .signAndSend(charleth, { tip: HIGH_TIP });

        const result = await context.createBlock();
        const hash = result.block.hash;
        const { block } = await context.polkadotJs().rpc.chain.getBlock(hash);

        // filter out inherent extrinsics, which should leave us with the ones we sent in their
        // inclusion order
        const transferExts = block.extrinsics.filter(
          (ext) => ext.signer.toHex() !== "0x0000000000000000000000000000000000000000"
        );

        expect(transferExts.length).to.eq(3);
        expect(transferExts[0].tip.toBigInt()).to.eq(HIGH_TIP);
        expect(transferExts[1].tip.toBigInt()).to.eq(MED_TIP);
        expect(transferExts[2].tip.toBigInt()).to.eq(NO_TIP);
      },
    });

    it({
      id: "T02",
      title: "should treat eth and substrate txns fairly",
      test: async function () {
        // tip 1 and 3 will be substrate txns, we express their tip above as per-gas but must send
        // them expressed as a flat tip. So we query the weight and convert to gas, then multiply
        // by the per-gas tip. We do this because it's the inverse of the txpool algo, and we want
        // to show that this algo will order txns by tip in this test.
        //
        // so the expected order is:
        // tip_0 (eth)
        // tip_1 (substrate)
        // tip_2 (eth)
        // tip_3 (substrate)
        const TIP_PER_GAS_0 = 10000n;
        const TIP_PER_GAS_1 = 20000n;
        const TIP_PER_GAS_2 = 30000n;
        const TIP_PER_GAS_3 = 40000n;

        // here we query the weight of a substrate balance transfer
        const dummyTransfer = context
          .polkadotJs()
          .tx.balances.transferAllowDeath(alith.address, GLMR);
        const info = await context
          .polkadotJs()
          .call.transactionPaymentApi.queryInfo(dummyTransfer.toHex(), dummyTransfer.encodedLength);
        const weight = info.weight.refTime.toBigInt();
        const balances_transfer_effective_gas = weight / WEIGHT_PER_GAS;

        // tx0 is an eth txn
        const tx0 = await createRawTransfer(context, ETHAN_ADDRESS, 1, {
          privateKey: CHARLETH_PRIVATE_KEY,
          maxFeePerGas: HIGH_MAX_FEE_PER_GAS,
          maxPriorityFeePerGas: TIP_PER_GAS_0,
        });

        // tx1 is a substrate txn
        const tx1 = await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(ethan.address, GLMR)
          .signAsync(alith, { tip: TIP_PER_GAS_1 * balances_transfer_effective_gas });

        // tx2 is an eth txn
        const tx2 = await createRawTransfer(context, ETHAN_ADDRESS, 1, {
          privateKey: DOROTHY_PRIVATE_KEY,
          maxFeePerGas: HIGH_MAX_FEE_PER_GAS,
          maxPriorityFeePerGas: TIP_PER_GAS_2,
        });

        // tx3 is a substrate txn
        const tx3 = await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(ethan.address, GLMR)
          .signAsync(baltathar, { tip: TIP_PER_GAS_3 * balances_transfer_effective_gas });

        const result = await context.createBlock([
          // use an order other than by priority
          tx2,
          tx3,
          tx0,
          tx1,
        ]);

        // get and filter the block's extrinsics
        const hash = result.block.hash;
        const { block } = await context.polkadotJs().rpc.chain.getBlock(hash);
        const transferExts = block.extrinsics.filter((ext) => {
          return (
            (ext.method.section == "balances" && ext.method.method == "transferAllowDeath") ||
            (ext.method.section == "ethereum" && ext.method.method == "transact")
          );
        });

        expect(transferExts.length).to.eq(4);
        expect(transferExts[0].method.section).to.eq("balances");
        expect(transferExts[1].method.section).to.eq("ethereum");
        expect(transferExts[2].method.section).to.eq("balances");
        expect(transferExts[3].method.section).to.eq("ethereum");
      },
    });

    it({
      id: "T03",
      title: "should allow Substrate txn replacement with higher priority",
      test: async function () {
        const LOW_TIP = 10n * MILLIGLMR;
        const HIGH_TIP = 20n * MILLIGLMR;

        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(dorothy.address, GLMR)
          .signAndSend(alith, { tip: LOW_TIP, nonce });

        await context
          .polkadotJs()
          .tx.system.remark("")
          .signAndSend(alith, { tip: HIGH_TIP, nonce });

        const result = await context.createBlock();
        const hash = result.block.hash;
        const { block } = await context.polkadotJs().rpc.chain.getBlock(hash);

        // filter out inherent extrinsics, which should leave us with the ones we sent in their
        // inclusion order
        const txnExts = block.extrinsics.filter(
          (ext) => ext.signer.toHex() !== "0x0000000000000000000000000000000000000000"
        );

        expect(txnExts.length).to.eq(1);
        expect(txnExts[0].tip.toBigInt()).to.eq(HIGH_TIP);
      },
    });

    it({
      id: "T04",
      title: "should allow Ethereum txn replacement with higher priority",
      test: async function () {
        const LOW_TIP = 10n * MILLIGLMR;
        const HIGH_TIP = 20n * MILLIGLMR;

        const randomAccount = generateKeyringPair();
        const randomAccount2 = generateKeyringPair();

        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

        // create a txn we don't expect to execute (because it will be replaced). it would send
        // some funds to randomAccount
        await sendRawTransaction(
          context,
          await createRawTransfer(context, randomAccount.address as `0x${string}`, 1, {
            maxFeePerGas: HIGH_MAX_FEE_PER_GAS,
            maxPriorityFeePerGas: LOW_TIP,
            nonce,
          })
        );

        // replace with a transaction that sends funds to a different account
        await sendRawTransaction(
          context,
          await createRawTransfer(context, randomAccount2.address as `0x${string}`, 1, {
            maxFeePerGas: HIGH_MAX_FEE_PER_GAS,
            maxPriorityFeePerGas: HIGH_TIP,
            nonce,
          })
        );

        await context.createBlock();

        const account1Balance = (
          await context.polkadotJs().query.system.account(randomAccount.address.toString())
        ).data.free.toBigInt();
        const account2Balance = (
          await context.polkadotJs().query.system.account(randomAccount2.address.toString())
        ).data.free.toBigInt();

        expect(account1Balance).to.eq(0n);
        expect(account2Balance).to.eq(1n);
      },
    });

    it({
      id: "T05",
      title: "should allow Ethereum txn replacement with Substrate txn",
      test: async function () {
        const randomAccount = generateKeyringPair();
        const randomAccount2 = generateKeyringPair();

        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

        // create a txn we don't expect to execute (because it will be replaced). it would send
        // some funds to randomAccount
        await sendRawTransaction(
          context,
          await createRawTransfer(context, randomAccount.address as `0x${string}`, 1, {
            nonce,
          })
        );

        // replace with a transaction that sends funds to a different account
        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(randomAccount2.address, 1)
          .signAndSend(alith, { nonce, tip: GLMR });

        await context.createBlock();

        const account1Balance = (
          await context.polkadotJs().query.system.account(randomAccount.address.toString())
        ).data.free.toBigInt();
        const account2Balance = (
          await context.polkadotJs().query.system.account(randomAccount2.address.toString())
        ).data.free.toBigInt();

        expect(account1Balance).to.eq(0n);
        expect(account2Balance).to.eq(1n);
      },
    });

    it({
      id: "T06",
      title: "should allow Substrate txn replacement with Ethereum txn",
      test: async function () {
        const randomAccount = generateKeyringPair();
        const randomAccount2 = generateKeyringPair();

        const nonce = await context.viem().getTransactionCount({ address: ALITH_ADDRESS });

        // create a txn we don't expect to execute (because it will be replaced). it would send
        // some funds to randomAccount
        await context
          .polkadotJs()
          .tx.balances.transferAllowDeath(randomAccount.address, 1)
          .signAndSend(alith, { nonce, tip: 0 });

        // replace with a transaction that sends funds to a different account
        await sendRawTransaction(
          context,
          await createRawTransfer(context, randomAccount2.address as `0x${string}`, 1, {
            maxFeePerGas: HIGH_MAX_FEE_PER_GAS,
            nonce,
          })
        );

        const result = await context.createBlock();

        const account1Balance = (
          await context.polkadotJs().query.system.account(randomAccount.address.toString())
        ).data.free.toBigInt();
        const account2Balance = (
          await context.polkadotJs().query.system.account(randomAccount2.address.toString())
        ).data.free.toBigInt();

        expect(account1Balance).to.eq(0n);
        expect(account2Balance).to.eq(1n);
      },
    });
  },
});
