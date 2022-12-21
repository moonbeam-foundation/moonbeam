import { ALITH_PRIVATE_KEY } from "../util/accounts";
import { expect } from "chai";
import { Contract, ethers, Wallet } from "ethers";
import {
  comptrollerAbi,
  dotMoneyMarketVaultAddress,
  fraxAbi,
  fraxAddress,
  fraxMoneyMarketVaultAddress,
  glmrMoneyMarketVaultAddress,
  mmGlmrVaultAbi,
  mmVaultAbi,
  moneyMarketComptrollerAddress,
  rewardToken,
  xcdotAddress,
  xcTokenAbi,
} from "./staticData";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
import { BASE_PATH, CUSTOM_SPEC_PATH, DEBUG_MODE } from "../util/constants";

const debug = require("debug")("contract-simulation:MM");

if (!CUSTOM_SPEC_PATH && !DEBUG_MODE) {
  console.error(`Missing CUSTOM_SPEC_PATH var`);
  console.log("Please provide path to modified chainSpec.");
  console.log("Alternatively, run in DEBUG_MODE to connect to existing local network.");
  process.exit(1);
}

if (!BASE_PATH && !DEBUG_MODE) {
  console.error(`Missing BASE_PATH var`);
  console.log("Please provide path to already setup node base folder.");
  console.log("Alternatively, run in DEBUG_MODE to connect to existing local network.");
  process.exit(1);
}

describeDevMoonbeam(
  `When interacting with MoneyMarket contracts on forked Moonbeam network...`,
  (context) => {
    let signer: Wallet;

    let comptrollerContract: Contract,
      dotVaultContract: Contract,
      fraxContract: Contract,
      fraxVaultContract: Contract,
      dotContract: Contract,
      glmrVaultContract: Contract,
      rewardTokenContract: Contract;

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
      glmrVaultContract = new ethers.Contract(glmrMoneyMarketVaultAddress, mmGlmrVaultAbi, signer);
      rewardTokenContract = new ethers.Contract(rewardToken, xcTokenAbi, signer);

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
        await context.createBlock();
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
      await dotVaultContract.mint(ethers.utils.parseUnits("5", await dotContract.decimals()), {
        gasLimit: "800000",
      });
      debug(`ℹ️  Depositing DOT into vault ...`);
      await context.createBlock();
      const dotBalanceAfter = await dotContract.balanceOf(signer.address);
      expect(dotBalanceBefore.gt(dotBalanceAfter), "DOT not deposited").to.be.true;

      if (
        (await comptrollerContract.checkMembership(signer.address, dotVaultContract.address)) &&
        (await comptrollerContract.checkMembership(signer.address, fraxVaultContract.address))
      ) {
        debug(`✅ DOT / FRAX has already been setup, skipping market admission`);
      } else {
        debug(`ℹ️  Entering DOT / FRAX markets, please wait ...`);
        await comptrollerContract.enterMarkets([
          dotVaultContract.address,
          fraxVaultContract.address,
        ]);
        await context.createBlock();
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
      debug(`ℹ️  Borrowing FRAX from protocol ...`);
      await context.createBlock();
      const fraxBalanceAfter = await fraxContract.balanceOf(signer.address);
      expect(fraxBalanceBefore.lt(fraxBalanceAfter), "Borrowed tokens not received").to.be.true;

      // pay back frax
      await fraxVaultContract.repayBorrow(
        ethers.utils.parseUnits("2", await fraxContract.decimals()),
        { gasLimit: "800000" }
      );
      debug(`ℹ️  Repaying FRAX back to protocol ...`);
      await context.createBlock();
      const fraxBalanceFinally = await fraxContract.balanceOf(signer.address);
      expect(fraxBalanceAfter.gt(fraxBalanceFinally), "FRAX has not been repaid").to.be.true;

      // remove dot
      await dotVaultContract.redeem(ethers.utils.parseUnits("1", await dotContract.decimals()), {
        gasLimit: "800000",
      });
      debug(`ℹ️  Removing DOT from vault ...`);
      await context.createBlock();
      const dotBalanceFinally = await dotContract.balanceOf(signer.address);
      expect(dotBalanceAfter.lt(dotBalanceFinally), "DOT not withdrawn").to.be.true;
    });

    it("should be able to add GLMR, harvest, remove", async function () {
      this.slow(120000);
      this.timeout(360000);

      // Depositing GLMR into vault
      const depositedGlmrBalanceBefore = await glmrVaultContract.balanceOf(signer.address);
      await glmrVaultContract.mint({
        value: ethers.utils.parseEther("10"),
        gasLimit: "800000",
      });
      debug(`ℹ️  Depositing GLMR into vault ...`);
      await context.createBlock();
      const depositedGlmrBalanceAfter = await glmrVaultContract.balanceOf(signer.address);
      expect(
        depositedGlmrBalanceBefore.lt(depositedGlmrBalanceAfter),
        "GLMR not deposited into vault"
      ).to.be.true;

      // Generating blocks to accrue rewards
      for (let i = 0; i < 500; i++) {
        await context.createBlock();
      }

      // Harvesting emitted rewards
      const harvestAmountBefore = await comptrollerContract.rewardAccrued(0, signer.address);
      const rewardTokenBalanceBefore = await rewardTokenContract.balanceOf(signer.address);
      await comptrollerContract["claimReward(uint8,address)"](0, signer.address, {
        gasLimit: "1400000",
      });

      debug(`ℹ️  Harvesting rewards ...`);
      await context.createBlock();
      const harvestAmountAfter = await comptrollerContract.rewardAccrued(0, signer.address);
      const rewardTokenBalanceAfter = await rewardTokenContract.balanceOf(signer.address);
      expect(harvestAmountBefore.gt(harvestAmountAfter), "No rewards harvested").to.be.true;
      expect(rewardTokenBalanceBefore.lt(rewardTokenBalanceAfter), "No reward tokens received").to
        .be.true;

      // Removing GLMR from vault
      await glmrVaultContract.redeem(ethers.utils.parseEther("10"), {
        gasLimit: "800000",
      });
      debug(`ℹ️  Removing GLMR from vault ...`);
      await context.createBlock();
      const depositedGlmrBalanceFinally = await dotContract.balanceOf(signer.address);
      expect(depositedGlmrBalanceAfter.lt(depositedGlmrBalanceFinally), "GLMR not withdrawn").to.be
        .true;
    });
  },
  null,
  null,
  null,
  true
);
