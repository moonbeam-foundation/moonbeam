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
    const signer = new ethers.Wallet(ALITH_PRIVATE_KEY, context.ethers);

    let usdtContract: Contract,
      poolContract: Contract,
      routerContract: Contract,
      dotContract: Contract,
      wglmrContract: Contract;

    before(async () => {
      dotContract = new ethers.Contract(xcdotAddress, xcTokenAbi, context.ethers);
      usdtContract = new ethers.Contract(usdtAddress, xcTokenAbi, context.ethers);
      wglmrContract = new ethers.Contract(wethAddress, wethAbi, context.ethers);
      poolContract = new ethers.Contract(glmrDotPoolAddress, glmrDotPoolAbi, context.ethers);
      routerContract = new ethers.Contract(ammRouterAddress, ammRouterAbi, context.ethers);

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

    // TODO: can calculate the amounts in and out of pool (2x test cases)

    // TODO: write test to swap some tokens

    // TODO: write test to add liquidity

    // TODO: write test to remove liquidity

    // TODO: Deposit LP token to farm

    // TODO: Harvest rewards from farm

    // TODO: Withdraw LP token from farm
  }
);
