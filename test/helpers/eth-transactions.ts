import "@moonbeam-network/api-augment";
import { type DevModeContext, expect } from "@moonwall/cli";
import type { EventRecord } from "@polkadot/types/interfaces";
import type {
  EvmCoreErrorExitError,
  EvmCoreErrorExitFatal,
  EvmCoreErrorExitReason,
  EvmCoreErrorExitRevert,
  EvmCoreErrorExitSucceed,
} from "@polkadot/types/lookup";
import assert from "node:assert";
import { fromHex } from "viem";

export type Errors = {
  Succeed: EvmCoreErrorExitSucceed["type"];
  Error: EvmCoreErrorExitError["type"];
  Revert: EvmCoreErrorExitRevert["type"];
  Fatal: EvmCoreErrorExitFatal["type"];
};

export async function extractRevertReason(context: DevModeContext, responseHash: string) {
  const tx = await context.ethers().provider?.getTransaction(responseHash);

  assert(tx, "Transaction not found");

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
  expect(events, "Missing events, probably failed execution").to.be.length.at.least(1);
  const ethereumResult = events.find(
    ({ event: { section, method } }) => section === "ethereum" && method === "Executed"
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

export function getSignatureParameters(signature: string) {
  const r = signature.slice(0, 66); // 32 bytes
  const s = `0x${signature.slice(66, 130)}`; // 32 bytes
  let v = fromHex(`0x${signature.slice(130, 132)}`, "number"); // 1 byte

  if (![27, 28].includes(v)) v += 27; // not sure why we coerce 27

  return {
    r,
    s,
    v,
  };
}
