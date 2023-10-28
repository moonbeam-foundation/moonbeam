import "@moonbeam-network/api-augment";
import { DevModeContext, expect } from "@moonwall/cli";
import { EventRecord } from "@polkadot/types/interfaces";
import {
  EvmCoreErrorExitError,
  EvmCoreErrorExitFatal,
  EvmCoreErrorExitReason,
  EvmCoreErrorExitRevert,
  EvmCoreErrorExitSucceed,
} from "@polkadot/types/lookup";
export type Errors = {
  Succeed: EvmCoreErrorExitSucceed["type"];
  Error: EvmCoreErrorExitError["type"];
  Revert: EvmCoreErrorExitRevert["type"];
  Fatal: EvmCoreErrorExitFatal["type"];
};

export async function extractRevertReason(context: DevModeContext, responseHash: string) {
  const tx = (await context.ethers().provider!.getTransaction(responseHash))!;
  try {
    await context.ethers().call({ to: tx.to, data: tx.data, gasLimit: tx.gasLimit });
    return null;
  } catch (e: any) {
    const errorMessage = e.info.error.message;
    return errorMessage.split("VM Exception while processing transaction: revert ")[1];
  }
}

export function expectEVMResult<T extends Errors, Type extends keyof T>(
  events: EventRecord[],
  resultType: Type,
  reason?: T[Type]
) {
  expect(events, `Missing events, probably failed execution`).to.be.length.at.least(1);
  const ethereumResult = events.find(
    ({ event: { section, method } }) => section == "ethereum" && method == "Executed"
  )!.event.data[3] as EvmCoreErrorExitReason;

  const foundReason = ethereumResult.isError
    ? ethereumResult.asError.type
    : ethereumResult.isFatal
    ? ethereumResult.asFatal.type
    : ethereumResult.isRevert
    ? ethereumResult.asRevert.type
    : ethereumResult.asSucceed.type;

  expect(
    ethereumResult.type,
    `Invalid EVM Execution - (${ethereumResult.type}.${foundReason})`
  ).to.equal(resultType);
  if (reason) {
    if (ethereumResult.isError) {
      expect(
        ethereumResult.asError.type,
        `Invalid EVM Execution ${ethereumResult.type} Reason`
      ).to.equal(reason);
    } else if (ethereumResult.isFatal) {
      expect(
        ethereumResult.asFatal.type,
        `Invalid EVM Execution ${ethereumResult.type} Reason`
      ).to.equal(reason);
    } else if (ethereumResult.isRevert) {
      expect(
        ethereumResult.asRevert.type,
        `Invalid EVM Execution ${ethereumResult.type} Reason`
      ).to.equal(reason);
    } else
      expect(
        ethereumResult.asSucceed.type,
        `Invalid EVM Execution ${ethereumResult.type} Reason`
      ).to.equal(reason);
  }
}

export async function getTransactionFees(context: DevModeContext, hash: string): Promise<bigint> {
  const receipt = await context.viem().getTransactionReceipt({ hash: hash as `0x${string}` });

  return receipt.gasUsed * receipt.effectiveGasPrice;
}
