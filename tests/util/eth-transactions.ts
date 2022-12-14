import { expect } from "chai";
import { EventRecord } from "@polkadot/types/interfaces";
import {
  EvmCoreErrorExitReason,
  EvmCoreErrorExitSucceed,
  EvmCoreErrorExitError,
  EvmCoreErrorExitRevert,
  EvmCoreErrorExitFatal,
} from "@polkadot/types/lookup";

export type Errors = {
  Succeed: EvmCoreErrorExitSucceed["type"];
  Error: EvmCoreErrorExitError["type"];
  Revert: EvmCoreErrorExitRevert["type"];
  Fatal: EvmCoreErrorExitFatal["type"];
};

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
