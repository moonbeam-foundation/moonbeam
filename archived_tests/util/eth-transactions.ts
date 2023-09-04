import { expect } from "chai";
import { EventRecord } from "@polkadot/types/interfaces";
import {
  EvmCoreErrorExitReason,
  EvmCoreErrorExitSucceed,
  EvmCoreErrorExitError,
  EvmCoreErrorExitRevert,
  EvmCoreErrorExitFatal,
} from "@polkadot/types/lookup";
import { ethers } from "ethers";

export type Errors = {
  Succeed: EvmCoreErrorExitSucceed["type"];
  Error: EvmCoreErrorExitError["type"];
  Revert: EvmCoreErrorExitRevert["type"];
  Fatal: EvmCoreErrorExitFatal["type"];
};

export async function extractRevertReason(
  responseHash: string,
  ethers: ethers.providers.JsonRpcProvider
) {
  const tx = await ethers.getTransaction(responseHash);
  try {
    await ethers.call(tx, tx.blockNumber);
    return null;
  } catch (e) {
    const jsonError = JSON.parse(e.error.body);
    return jsonError.error.message.split("VM Exception while processing transaction: revert ")[1];
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
  ).event.data[3] as EvmCoreErrorExitReason;

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
