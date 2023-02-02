import { ALITH_PRIVATE_KEY } from "../util/accounts";
import { expect } from "chai";
import { BigNumber, Contract, ethers, Wallet } from "ethers";
import {
  ammRouterAbi,
  ammRouterAddress,
  glmrDotPoolAbi,
  glmrDotPoolAddress,
  usdtAddress,
  wethAbi,
  wethAddress,
  xcdotAddress,
  xcTokenAbi,
} from "./staticData";

import { BASE_PATH, CUSTOM_SPEC_PATH, DEBUG_MODE, TWO_MINS } from "../util/constants";
import { describeDevMoonbeam } from "../util/setup-dev-tests";
const RUNTIME_NAME = process.env.RUNTIME_NAME as "moonbeam" | "moonbase" | "moonriver";
const SPEC_FILE = process.env.SPEC_FILE;
const ROUNDS_TO_WAIT = (process.env.ROUNDS_TO_WAIT && parseInt(process.env.ROUNDS_TO_WAIT)) || 2;
const PARA_ID = process.env.PARA_ID && parseInt(process.env.PARA_ID);
const SKIP_INTERMEDIATE_RUNTIME = process.env.SKIP_INTERMEDIATE_RUNTIME == "true";

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

const debug = require("debug")("contract-simulation:AMM");

describeDevMoonbeam(
  `When interacting with AMM contracts on forked Moonbeam network...`,
  (context) => {
    let signer: Wallet;

    let usdtContract: Contract,
      poolContract: Contract,
      routerContract: Contract,
      dotContract: Contract,
      wglmrContract: Contract;

    before(async () => {
      signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);
      signer.connect(context.ethers);

      dotContract = new ethers.Contract(xcdotAddress, xcTokenAbi, signer);
      usdtContract = new ethers.Contract(usdtAddress, xcTokenAbi, signer);
      wglmrContract = new ethers.Contract(wethAddress, wethAbi, signer);
      poolContract = new ethers.Contract(glmrDotPoolAddress, glmrDotPoolAbi, signer);
      routerContract = new ethers.Contract(ammRouterAddress, ammRouterAbi, signer);

      const alithUsdtBalance = await usdtContract.balanceOf(signer.address);
      const alithDotBalance = await dotContract.balanceOf(signer.address);
      const alithGlmrBalance = await context.ethers.getBalance(signer.address);

      debug(
        `Alith Balances - ${ethers.utils.formatEther(
          alithGlmrBalance
        )} GLMR, ${ethers.utils.formatUnits(
          alithUsdtBalance,
          await usdtContract.decimals()
        )} USDT, ${ethers.utils.formatUnits(alithDotBalance, await dotContract.decimals())} DOT`
      );

      expect(
        [alithUsdtBalance.isZero(), alithDotBalance.isZero(), alithGlmrBalance.isZero()],
        "❌ Balances not injected into Alith Account"
      ).to.not.include(true);

      // Perform token approvals
      const dotApprovalAmount = await dotContract.allowance(signer.address, routerContract.address);
      const usdtApprovalAmount = await usdtContract.allowance(
        signer.address,
        routerContract.address
      );
      const poolApprovalAmount = await poolContract.allowance(
        signer.address,
        routerContract.address
      );

      if (
        dotApprovalAmount.isZero() ||
        usdtApprovalAmount.isZero() ||
        poolApprovalAmount.isZero()
      ) {
        await dotContract.approve(routerContract.address, ethers.constants.MaxUint256);
        await usdtContract.approve(routerContract.address, ethers.constants.MaxUint256);
        await poolContract.approve(routerContract.address, ethers.constants.MaxUint256);
        debug(`ℹ️  Setting allowances, please wait ...`);
        await context.createBlock();
        const dotApprovalAmount = await dotContract.allowance(
          signer.address,
          routerContract.address
        );
        const usdtApprovalAmount = await usdtContract.allowance(
          signer.address,
          routerContract.address
        );
        const poolApprovalAmount = await poolContract.allowance(
          signer.address,
          routerContract.address
        );
        expect(
          [dotApprovalAmount.isZero(), usdtApprovalAmount.isZero(), poolApprovalAmount.isZero()],
          "Approval amount has not been increased"
        ).to.not.include(true);
      } else {
        debug(`✅ Allowances already set, skipping approvals`);
      }
    });

    it("...should have DOT/WGLMR token balances in pool", async function () {
      const dotPoolBalance = await dotContract.balanceOf(poolContract.address);
      const wglmrPoolBalance = await wglmrContract.balanceOf(poolContract.address);
      expect([dotPoolBalance.isZero(), wglmrPoolBalance.isZero()]).to.not.include(true);
    });

    it("...should have expected token addresses for pool", async function () {
      const token0Address = await poolContract.token0();
      const token1Address = await poolContract.token1();
      expect([
        token0Address.toLowerCase() === wglmrContract.address.toLowerCase(),
        token1Address.toLowerCase() === dotContract.address.toLowerCase(),
      ]).to.not.include(false);
    });

    it("...should have matching WETH address for router", async function () {
      expect(
        (await routerContract.WETH()).toLowerCase() === wethAddress.toLowerCase(),
        `❌ WETH address is not match, is this the right network? (moonbeam)`
      ).to.be.true;
    });

    it("...should calculate swap amount out", async function () {
      const calculatedAmount = await routerContract.getAmountsOut(ethers.utils.parseEther("1"), [
        wethAddress,
        usdtAddress,
      ]);
      debug(
        `Calculated that 1 GLMR can be swapped for ${ethers.utils.formatUnits(
          calculatedAmount[1],
          await usdtContract.decimals()
        )} USDT`
      );
      expect(calculatedAmount[1].isZero()).to.not.be.true;
    });

    it("...should be able to swap tokens.", async function () {
      this.slow(30000);
      this.timeout(TWO_MINS);
      const dotBalanceBefore = await dotContract.balanceOf(signer.address);
      const systemBalanceBefore = await signer.getBalance();

      const deadline = Math.floor(Number(Date.now()) / 1000) + 3000;
      await routerContract.swapExactETHForTokens(
        ethers.utils.parseUnits("0.1", await dotContract.decimals()),
        [wglmrContract.address, dotContract.address],
        signer.address,
        deadline,
        { value: ethers.utils.parseEther("100"), gasLimit: "200000" }
      );
      debug(`ℹ️  Swapping GLMR for DOT ...`);
      await context.createBlock();

      const dotBalanceAfter = await dotContract.balanceOf(signer.address);
      const systemBalanceAfter = await signer.getBalance();
      debug(
        `Alith Balances before: ${ethers.utils.formatUnits(
          dotBalanceBefore,
          await dotContract.decimals()
        )} DOT, ${ethers.utils.formatEther(
          systemBalanceBefore
        )}  GLMR; after:  ${ethers.utils.formatUnits(
          dotBalanceAfter,
          await dotContract.decimals()
        )} DOT, ${ethers.utils.formatEther(systemBalanceAfter)} GLMR`
      );
      expect(
        [dotBalanceBefore.lt(dotBalanceAfter), systemBalanceBefore.gt(systemBalanceAfter)],
        "Balances post-swap have not been updated"
      ).to.not.include(false);
    });

    it("...should be able to add/remove liquidity to the pool", async function () {
      this.slow(50000);
      this.timeout(TWO_MINS);

      const poolTokenBalanceBefore = await poolContract.balanceOf(signer.address);
      const dotTokenBalanceBefore = await dotContract.balanceOf(signer.address);
      const glmrTokenBalanceBefore = await signer.getBalance();
      const glmrAmount = ethers.utils.parseEther("130");
      const dotAmount = ethers.utils.parseUnits("10", await dotContract.decimals());

      /// Add liquidity
      await routerContract.addLiquidityETH(
        dotContract.address,
        dotAmount,
        BigNumber.from(0),
        BigNumber.from(0),
        signer.address,
        Math.floor(Number(Date.now()) / 1000) + 3000,
        { value: glmrAmount, gasLimit: "300000" }
      );
      debug(`ℹ️  Adding liquidity to pool ...`);
      await context.createBlock();

      /// baalnces again
      const poolTokenBalanceAfter = await poolContract.balanceOf(signer.address);
      const dotTokenBalanceAfter = await dotContract.balanceOf(signer.address);
      const glmrTokenBalanceAfter = await signer.getBalance();

      expect(
        [
          poolTokenBalanceBefore.lt(poolTokenBalanceAfter),
          dotTokenBalanceBefore.gt(dotTokenBalanceAfter),
          glmrTokenBalanceBefore.gt(glmrTokenBalanceAfter),
        ],
        "Balances have not been updated after AddLiquidityETH call"
      ).to.not.include(false);

      // Remove Liquidity
      await routerContract.removeLiquidityETH(
        dotContract.address,
        poolTokenBalanceAfter,
        BigNumber.from(0),
        BigNumber.from(0),
        signer.address,
        Math.floor(Number(Date.now()) / 1000) + 3000,
        { gasLimit: "300000" }
      );
      debug(`ℹ️  Removing liquidity from pool ...`);
      await context.createBlock();

      const poolTokenBalanceFinally = await poolContract.balanceOf(signer.address);
      const dotTokenBalanceFinally = await dotContract.balanceOf(signer.address);
      const glmrTokenBalanceFinally = await signer.getBalance();

      expect(
        [
          poolTokenBalanceFinally.lt(poolTokenBalanceAfter),
          dotTokenBalanceFinally.gt(dotTokenBalanceAfter),
          glmrTokenBalanceFinally.gt(glmrTokenBalanceAfter),
        ],
        "Balances have not been updated after RemoveLiquidityETH call"
      ).to.not.include(false);
    });
  },
  null,
  null,
  null,
  true
);
