import "@moonbeam-network/api-augment";
import { DevModeContext } from "@moonwall/cli";
import { GLMR, KeyringPair, MIN_GLMR_STAKING, alith, generateKeyringPair } from "@moonwall/util";
import chalk from "chalk";
import { Debugger } from "debug";

export async function createAccounts(
  context: DevModeContext,
  maxAccounts: number
): Promise<KeyringPair[]> {
  const randomAccounts = new Array(Number(maxAccounts)).fill(0).map(() => generateKeyringPair());

  let alithNonce = await context
    .viem()
    .getTransactionCount({ address: alith.address as `0x${string}` });
  await context.createBlock(
    randomAccounts.map((randomCandidate) =>
      context
        .polkadotJs()
        .tx.sudo.sudo(
          context
            .polkadotJs()
            .tx.balances.forceSetBalance(
              randomCandidate.address,
              12n * MIN_GLMR_STAKING + 50n * GLMR
            )
        )
        .signAsync(alith, { nonce: alithNonce++ })
    ),
    { allowFailures: false }
  );

  return randomAccounts;
}

export async function countExtrinsics(
  context: DevModeContext,
  method: string,
  logger: Debugger
): Promise<[number, number, number]> {
  const block = await context.polkadotJs().rpc.chain.getBlock();
  const extrinsicCount = block.block.extrinsics.reduce(
    (acc, ext) =>
      acc + (ext.method.section === "parachainStaking" && ext.method.method === method ? 1 : 0),
    0
  );

  const maxBlockWeights = context.polkadotJs().consts.system.blockWeights;
  const blockWeights = await context.polkadotJs().query.system.blockWeight();

  const weightUtil =
    blockWeights.normal.refTime.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().refTime.toNumber();
  const proofUtil =
    blockWeights.normal.proofSize.toNumber() /
    maxBlockWeights.perClass.normal.maxTotal.unwrap().proofSize.toNumber();

  logger(
    `  ${chalk.yellow("○")} ${chalk.gray(method)} max ${chalk.green(
      extrinsicCount
    )} per block (w: ${(weightUtil * 100).toFixed(1)}%, p: ${(proofUtil * 100).toFixed(1)}%)`
  );

  return [extrinsicCount, weightUtil, proofUtil];
}
