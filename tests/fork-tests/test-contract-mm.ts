import { ALITH_PRIVATE_KEY } from "../util/accounts";
import { expect } from "chai";
import { BigNumber, Contract, ethers } from "ethers";
import {
  comptrollerAbi,
  dotMoneyMarketVaultAddress,
  fraxAbi,
  fraxAddress,
  fraxMoneyMarketVaultAddress,
  mmVaultAbi,
  moneyMarketComptrollerAddress,
  xcdotAddress,
  xcTokenAbi,
} from "./staticData";

import { describeParachain } from "../util/setup-para-tests";
import { TWO_MINS } from "../util/constants";

const debug = require("debug")("contract-sim:amm");
const RUNTIME_NAME = process.env.RUNTIME_NAME as "moonbeam" | "moonbase" | "moonriver";
const SPEC_FILE = process.env.SPEC_FILE;
const PARA_ID = process.env.PARA_ID && parseInt(process.env.PARA_ID);

if (!RUNTIME_NAME) {
  console.error(`Missing RUNTIME_NAME (ex: moonbeam)`);
  process.exit(1);
}

if (!SPEC_FILE) {
  console.error(`Missing SPEC_FILE (ex: ~/exports/moonbeam-state.mod.json)`);
  process.exit(1);
}

if (!PARA_ID) {
  console.error(`Missing PARA_ID (ex: 2004)`);
  process.exit(1);
}

describeParachain(
  `When interacting with MoneyMarket contracts on forked ${RUNTIME_NAME} network...`,
  {
    parachain: {
      spec: SPEC_FILE,
      binary: "local",
    },
    paraId: PARA_ID,
    relaychain: {
      binary: "local",
    },
  },
  (context) => {
    let signer;

    let comptrollerContract: Contract,
      dotVaultContract: Contract,
      fraxContract: Contract,
      fraxVaultContract: Contract,
      dotContract: Contract;

    before(async () => {
      signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);
      signer.connect(context.ethers);

      comptrollerContract = new ethers.Contract(
        moneyMarketComptrollerAddress,
        comptrollerAbi,
        signer
      );
      dotVaultContract = new ethers.Contract(dotMoneyMarketVaultAddress, mmVaultAbi, signer);
      dotContract = new ethers.Contract(xcdotAddress, xcTokenAbi, signer);
      fraxContract = new ethers.Contract(fraxAddress, fraxAbi, signer);
      fraxVaultContract = new ethers.Contract(fraxMoneyMarketVaultAddress, mmVaultAbi, signer);

      const alithFraxBalance = await fraxContract.balanceOf(signer.address);
      const alithDotBalance = await dotContract.balanceOf(signer.address);
      const alithGlmrBalance = await context.ethers.getBalance(signer.address);

      debug(
        `Alith Balances - ${ethers.utils.formatEther(
          alithGlmrBalance
        )} GLMR, ${ethers.utils.formatUnits(
          alithFraxBalance,
          await fraxContract.decimals()
        )} FRAX, ${ethers.utils.formatUnits(alithDotBalance, await dotContract.decimals())} DOT`
      );

      expect(
        [alithDotBalance.isZero(), alithGlmrBalance.isZero()],
        "❌ Balances not injected into Alith Account"
      ).to.not.include(true);

      // Perform token approvals
      const dotApprovalAmount = await dotContract.allowance(
        signer.address,
        dotVaultContract.address
      );
      const fraxApprovalAmount = await fraxContract.allowance(
        signer.address,
        fraxVaultContract.address
      );

      if (dotApprovalAmount.isZero() || fraxApprovalAmount.isZero()) {
        await dotContract.approve(dotVaultContract.address, ethers.constants.MaxUint256);
        await fraxContract.approve(fraxVaultContract.address, ethers.constants.MaxUint256);
        debug(`ℹ️ Setting allowances, please wait ...`);
        await context.waitBlocks(2);
        const dotApprovalAmount = await dotContract.allowance(
          signer.address,
          dotVaultContract.address
        );
        const fraxApprovalAmount = await fraxContract.allowance(
          signer.address,
          fraxVaultContract.address
        );
        expect(
          [dotApprovalAmount.isZero(), fraxApprovalAmount.isZero()],
          "Approval amount has not been increased"
        ).to.not.include(true);
      } else {
        debug(`✅ Allowances already set, skipping approvals`);
      }
    });

    it("...should be able to add a token to vault and borrow another", async function () {
      this.slow(120000);
      this.timeout(360000);
      // Depositing xcDOT into vault
      const dotBalanceBefore = await dotContract.balanceOf(signer.address);
      await dotVaultContract.mint(ethers.utils.parseUnits("5", await dotContract.decimals()));
      debug(`ℹ️ Depositing DOT into vault ...`);
      await context.waitBlocks(2);
      const dotBalanceAfter = await dotContract.balanceOf(signer.address);
      expect(dotBalanceBefore.gt(dotBalanceAfter), "DOT not deposited").to.be.true;

      if (
        (await comptrollerContract.checkMembership(signer.address, dotVaultContract.address)) &&
        (await comptrollerContract.checkMembership(signer.address, fraxVaultContract.address))
      ) {
        debug(`✅ DOT / FRAX has already been setup, skipping market admission`);
      } else {
        debug(`ℹ️ Entering DOT / FRAX markets, please wait ...`);
        await comptrollerContract.enterMarkets([
          dotVaultContract.address,
          fraxVaultContract.address,
        ]);
        await context.waitBlocks(2);
        expect(
          await comptrollerContract.checkMembership(signer.address, dotVaultContract.address),
          "DOT market has not been entered"
        ).to.be.true;
      }

      // Borrowing FRAX against position
      const fraxBalanceBefore = await fraxContract.balanceOf(signer.address);
      await fraxVaultContract.borrow(ethers.utils.parseUnits("10", await fraxContract.decimals()), {
        gasLimit: "800000",
      });
      // await fraxVaultContract.approve(signer.address, fraxContract.address);
      debug(`ℹ️ Borrowing FRAX from protocol ...`);
      await context.waitBlocks(2);
      const fraxBalanceAfter = await fraxContract.balanceOf(signer.address);
      expect(fraxBalanceBefore.lt(fraxBalanceAfter), "Borrowed tokens not received").to.be.true;

      // pay back frax
      await fraxVaultContract.repayBorrow(
        ethers.utils.parseUnits("2", await fraxContract.decimals()),
        { gasLimit: "800000" }
      );
      debug(`ℹ️ Repaying FRAX back to protocol ...`);
      await context.waitBlocks(2);
      const fraxBalanceFinally = await fraxContract.balanceOf(signer.address);
      expect(fraxBalanceAfter.gt(fraxBalanceFinally), "FRAX has not been repaid").to.be.true;

      // remove dot
      await dotVaultContract.redeem(ethers.utils.parseUnits("1", await dotContract.decimals()), {
        gasLimit: "800000",
      });
      debug(`ℹ️ Removing DOT from vault ...`);
      await context.waitBlocks(2);
      const dotBalanceFinally = await dotContract.balanceOf(signer.address);
      expect(dotBalanceAfter.lt(dotBalanceFinally), "DOT not withdrawn").to.be.true;
    });

    // it should be able to add dot, borrow, payback

    // it should be able to add GLMR, harvest, remove
  }
);
