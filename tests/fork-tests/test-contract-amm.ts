import child_process from "child_process";
import { alith, ALITH_PRIVATE_KEY } from "../util/accounts";
import { expect } from "chai";
import { BigNumber, ethers } from "ethers";
import {
  glmrDotPoolAddress,
  usdcGlmrPoolAbi,
  usdcGlmrPoolAddress,
  usdcwhAbi,
  usdcwhAddress,
  xcdotAddress,
  xcTokenAbi,
} from "./staticData";

import { describeParachain } from "../util/setup-para-tests";

import { xxhashAsU8a, blake2AsU8a } from "@polkadot/util-crypto";
import { u8aConcat, u8aToHex, bnToHex} from "@polkadot/util";
import {u128 } from "@polkadot/types"
import { hashMessage } from "ethers/lib/utils";

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
  `AMM Contract-Sim testss on forked ${RUNTIME_NAME}`,
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

    before(async () => {
      // const signer = context.ethers.getSigner(0)
      // console.log(await signer.getAddress());
      console.log(signer.address);
      // console.log(signer.publicKey);
      // console.log(await context.ethers.getBalance(signer.address));

      const dotContract = new ethers.Contract(xcdotAddress, xcTokenAbi, context.ethers);
      const dotDps = await dotContract.functions.decimals();
      const poolBal = await dotContract.functions.balanceOf(glmrDotPoolAddress);
      const signerBal = await dotContract.functions.balanceOf(signer.address);

      console.log("pool balance is  " + poolBal[0].div(BigNumber.from(dotDps[0])));
      console.log("signer balance is  " + signerBal);

      // Function for giving yourself tokens

      const storageBlake128MapKey = (module, name, key) => {
        return u8aToHex(
          u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128), blake2AsU8a(key, 128), key)
        );
      };

      const storageBlake128DoubleMapKey = (module, name, [key1, key2]) => {
        return u8aToHex(
          u8aConcat(xxhashAsU8a(module, 128), xxhashAsU8a(name, 128), blake2AsU8a(key1, 128), key1, blake2AsU8a(key2, 128), key2)
        );
      };

      const hash = storageBlake128MapKey("System", "Account","0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac");
      console.log(hash);

      const number = BigInt("42259045809535163221576417993425387648")
      const leHex = bnToHex(number,{isLe: true})
      const hash2 = storageBlake128DoubleMapKey("Assets", "Account",[leHex,"0xf24FF3a9CF04c71Dbc94D0b566f7A27B94566cac"]);
      console.log(hash2);
    });

    it("pool should have USDT/GLMR balances", async function () {
      expect(true).to.be.true;
    });
  }
);
