import type { DevModeContext } from "@moonwall/cli";
import {
  BALTATHAR_PRIVATE_KEY,
  CHARLETH_PRIVATE_KEY,
  DOROTHY_PRIVATE_KEY,
  alith,
} from "@moonwall/util";
import type { PalletRandomnessRandomnessResult } from "@polkadot/types/lookup";
import { nToHex } from "@polkadot/util";
import { fromBytes, parseEther } from "viem";

export const RANDOMNESS_SOURCE_LOCAL_VRF = "0";
export const RANDOMNESS_SOURCE_BABE_EPOCH = "1";

export const setupLotteryWithParticipants = async (
  context: DevModeContext,
  source: "BABE" | "VRF"
) => {
  const { contractAddress: lotteryAddress } = await context.deployContract!(
    "RandomnessLotteryDemo",
    {
      args: [source === "BABE" ? RANDOMNESS_SOURCE_BABE_EPOCH : RANDOMNESS_SOURCE_LOCAL_VRF],
      value: parseEther("1"),
      gas: 5_000_000n,
    }
  );

  for (const privateKey of [DOROTHY_PRIVATE_KEY, BALTATHAR_PRIVATE_KEY, CHARLETH_PRIVATE_KEY]) {
    await context.writeContract!({
      contractName: "RandomnessLotteryDemo",
      contractAddress: lotteryAddress,
      functionName: "participate",
      args: [],
      gas: 500_000n,
      value: parseEther("1"),
      privateKey,
    });
  }
  await context.createBlock();
  return lotteryAddress;
};

// Uses sudo (alith) to set relayEpoch to +2 and randomnessResult to the desired value
export const fakeBabeResultTransaction = async (
  context: DevModeContext,
  value?: PalletRandomnessRandomnessResult
) => {
  const fakeRandomResult = context.polkadotJs().registry.createType(
    "Option<PalletRandomnessRandomnessResult>",
    value || {
      requestCount: 1,
      randomness: "0xb1ffdd4a26e0f2a2fd1e0862a1c9be422c66dddd68257306ed55dc7bd9dce647",
    }
  );

  return context
    .polkadotJs()
    .tx.sudo.sudo(
      context.polkadotJs().tx.system.setStorage([
        [
          context.polkadotJs().query.randomness.relayEpoch.key().toString(),
          nToHex((await context.polkadotJs().query.randomness.relayEpoch()).toBigInt() + 2n, {
            bitLength: 64,
            isLe: true,
          }),
        ],
        [
          context.polkadotJs().query.randomness.randomnessResults.key({ BabeEpoch: 2 }).toString(),
          fakeRandomResult.toHex(),
        ],
      ])
    )
    .signAsync(alith);
};

export const SIMPLE_SALT = fromBytes(
  new Uint8Array([..."my_salt".padEnd(32, " ")].map((a) => a.charCodeAt(0))),
  "hex"
);
