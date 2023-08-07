import { BlockCreationResponse, expect } from "@moonwall/cli";
import type { EventRecord } from "@polkadot/types/interfaces";
import {
  ApiTypes,
  AugmentedEvent,
  AugmentedEvents,
  SubmittableExtrinsic,
} from "@polkadot/api/types";
import { IEvent } from "@polkadot/types/types";

export type ExtractTuple<P> = P extends AugmentedEvent<"rxjs", infer T> ? T : never;

export async function expectOk<
  ApiType extends ApiTypes,
  Call extends
    | SubmittableExtrinsic<ApiType>
    | Promise<SubmittableExtrinsic<ApiType>>
    | string
    | Promise<string>,
  Calls extends Call | Call[],
  BlockCreation extends BlockCreationResponse<
    ApiType,
    Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>
  >
>(call: Promise<BlockCreation>): Promise<BlockCreation> {
  const block = await call;
  if (Array.isArray(block.result)) {
    block.result.forEach((r, idx) => {
      expect(
        r.successful,
        `tx[${idx}] - ${r.error?.name}${
          r.extrinsic
            ? `\n\t\t${r.extrinsic.method.section}.${r.extrinsic.method.method}(${r.extrinsic.args
                .map((d) => d.toHuman())
                .join("; ")})`
            : ""
        }`
      ).to.be.true;
    });
  } else {
    expect(block.result.successful, block.result.error?.name).to.be.true;
  }
  return block;
}

export function expectSubstrateEvent<
  ApiType extends ApiTypes,
  Call extends
    | SubmittableExtrinsic<ApiType>
    | Promise<SubmittableExtrinsic<ApiType>>
    | string
    | Promise<string>,
  Calls extends Call | Call[],
  Event extends AugmentedEvents<ApiType>,
  Section extends keyof Event,
  Method extends keyof Event[Section],
  Tuple extends ExtractTuple<Event[Section][Method]>
>(
  block: BlockCreationResponse<ApiType, Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>>,
  section: Section,
  method: Method
): IEvent<Tuple> {
  let event: EventRecord | undefined;
  if (Array.isArray(block.result)) {
    block.result.forEach((r, idx) => {
      const foundEvents = r.events.filter(
        ({ event }) => event.section.toString() == section && event.method.toString() == method
      );
      if (foundEvents.length > 0) {
        expect(
          event,
          `Event ${section.toString()}.${method.toString()} appeared multiple times`
        ).toBeUndefined();
        expect(
          foundEvents,
          `Event ${section.toString()}.${method.toString()} appeared multiple times`
        ).to.be.length(1);
        event = foundEvents[0];
      }
    });
  } else {
    const foundEvents = block.result!.events!.filter(
      ({ event }) => event.section.toString() == section && event.method.toString() == method
    );
    if (foundEvents.length > 0) {
      expect(
        foundEvents,
        `Event ${section.toString()}.${method.toString()} appeared multiple times`
      ).to.be.length(1);
      event = foundEvents[0];
    }
  }
  expect(
    event,
    `Event ${section.toString()}.${method.toString()} not found:\n${(Array.isArray(block.result)
      ? block.result.map((r) => r.events).flat()
      : block.result
      ? block.result.events
      : []
    )
      .map(({ event }) => `       - ${event.section.toString()}.${event.method.toString()}\n`)
      .join("")}`
  ).to.not.be.undefined;
  return event!.event as any;
}

export function expectSubstrateEvents<
  ApiType extends ApiTypes,
  Call extends
    | SubmittableExtrinsic<ApiType>
    | Promise<SubmittableExtrinsic<ApiType>>
    | string
    | Promise<string>,
  Calls extends Call | Call[],
  Event extends AugmentedEvents<ApiType>,
  Section extends keyof Event,
  Method extends keyof Event[Section],
  Tuple extends ExtractTuple<Event[Section][Method]>
>(
  block: BlockCreationResponse<ApiType, Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>>,
  section: Section,
  method: Method,
  count = 0 // if 0, doesn't check
): IEvent<Tuple>[] {
  let events: EventRecord[] = [];
  if (Array.isArray(block.result)) {
    block.result.forEach((r, idx) => {
      const foundEvents = r.events.filter(
        ({ event }) => event.section.toString() == section && event.method.toString() == method
      );
      if (foundEvents.length > 0) {
        events.push(...foundEvents);
      }
    });
  } else {
    const foundEvents = block.result.events.filter(
      ({ event }) => event.section.toString() == section && event.method.toString() == method
    );
    if (foundEvents.length > 0) {
      events.push(...foundEvents);
    }
  }
  expect(events.length > 0).to.not.be.null;
  return events.map(({ event }) => event) as any;
}
