import child_process from "child_process";
import { alith, ALITH_PRIVATE_KEY } from "../util/accounts";
import { expect } from "chai";
import { BigNumber, Contract, ethers } from "ethers";
import {
  ammRouterAbi,
  ammRouterAddress,
  glmrDotPoolAbi,
  glmrDotPoolAddress,
  usdcGlmrPoolAbi,
  usdcGlmrPoolAddress,
  usdcwhAbi,
  usdcwhAddress,
  usdtAddress,
  wethAbi,
  wethAddress,
  xcdotAddress,
  xcTokenAbi,
} from "./staticData";

import { describeParachain } from "../util/setup-para-tests";

import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex, bnToHex } from "@polkadot/util";
import { u128 } from "@polkadot/types";
import { hashMessage } from "ethers/lib/utils";
import { createImportSpecifier } from "typescript";

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
  `When interacting with AMM contracts on forked ${RUNTIME_NAME} network...`,
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

      const alithUsdtBalance = (await usdtContract.functions.balanceOf(signer.address))[0];
      const alithDotBalance = (await dotContract.functions.balanceOf(signer.address))[0];
      const alithGlmrBalance = await context.ethers.getBalance(signer.address);
      debug(
        `Alith Balances - USDT:${alithUsdtBalance.div(
          BigNumber.from((await usdtContract.functions.decimals())[0] ** 10)
        )} DOT:${alithDotBalance.div(
          BigNumber.from((await dotContract.functions.decimals())[0] ** 10)
        )} GLMR:${ethers.utils.formatEther(alithGlmrBalance)}`
      );

      expect(
        [alithUsdtBalance.isZero(), alithDotBalance.isZero(), alithGlmrBalance.isZero()],
        "❌ Balances not injected into Alith Account"
      ).to.not.include(true);
    });

    it("...should have DOT/WGLMR token balances in pool", async function () {
      const dotPoolBalance = await dotContract.functions.balanceOf(poolContract.address);
      const wglmrPoolBalance = await wglmrContract.functions.balanceOf(poolContract.address);
      expect([dotPoolBalance[0].isZero(), wglmrPoolBalance[0].isZero()]).to.not.include(true);
    });

    it("...should have expected token addresses for pool", async function () {
      const token0Address = await poolContract.functions.token0();
      const token1Address = await poolContract.functions.token1();
      expect([
        token0Address[0].toLowerCase() === wglmrContract.address.toLowerCase(),
        token1Address[0].toLowerCase() === dotContract.address.toLowerCase(),
      ]).to.not.include(false);
    });

    it("...should have matching WETH address for router", async function () {
      expect(
        (await routerContract.functions.WETH())[0].toLowerCase() === wethAddress.toLowerCase(),
        `❌ WETH address is not match, is this the right network? (moonbeam)`
      ).to.be.true;
    });

    it("...should calculate swap amount out", async function () {
      const calculatedAmount = await routerContract.functions.getAmountsOut(
        ethers.utils.parseEther("1"),
        [wethAddress, usdtAddress]
      );
      debug(
        `Calculated that 1 GLMR can be swapped for ${ethers.utils.formatUnits(
          calculatedAmount[0][1],
          await usdtContract.functions.decimals()
        )} USDT`
      );
      expect(calculatedAmount[0][1].isZero()).to.not.be.true;
    });

    it.only("...should be able to swap approved tokens.", async function () {
      await dotContract.functions.approve(routerContract.address, ethers.constants.MaxUint256);
      const approvalAmount = await dotContract.functions.allowance(
        signer.address,
        routerContract.address
      );
      await context.waitBlocks(1);
      expect(approvalAmount[0].isZero()).to.be.false;

      // console.log(await dotContract.balanceOf(signer.address))

      const dotBalanceBefore = await dotContract.functions.balanceOf(signer.address);
      const systemBalanceBefore = await signer.getBalance();

      const deadline = Math.floor(Number(Date.now()) / 1000) + 3000;
      await routerContract.functions.swapExactETHForTokens(
        ethers.utils.parseUnits("0.1", await dotContract.functions.decimals()),
        [wglmrContract.address, dotContract.address],
        signer.address,
        deadline,
        { value: ethers.utils.parseEther("100"), gasLimit: "200000" }
      );
      await context.waitBlocks(2);

      const dotBalanceAfter = await dotContract.functions.balanceOf(signer.address);
      const systemBalanceAfter = await signer.getBalance();
      debug(
        `Alith Balances before: ${ethers.utils.formatUnits(
          dotBalanceBefore[0],
          await dotContract.functions.decimals()
        )} DOT, ${ethers.utils.formatEther(
          systemBalanceBefore
        )}  GLMR; after:  ${ethers.utils.formatUnits(
          dotBalanceAfter[0],
          await dotContract.functions.decimals()
        )} DOT, ${ethers.utils.formatEther(systemBalanceAfter)} GLMR`
      );
      expect([dotBalanceBefore[0].lt(dotBalanceAfter[0]), systemBalanceBefore.gt(systemBalanceAfter)]).to.not.include(false)
    });

    // TODO: write test to add liquidity

    // TODO: write test to remove liquidity

    // TODO: Deposit LP token to farm

    // TODO: Harvest rewards from farm

    // TODO: Withdraw LP token from farm
  }
);
