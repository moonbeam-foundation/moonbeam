import { BlockCreationResponse } from "./setup-dev-tests";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { expect } from "chai";

export async function expectOk<
  ApiType extends ApiTypes,
  Call extends
    | SubmittableExtrinsic<ApiType>
    | Promise<SubmittableExtrinsic<ApiType>>
    | string
    | Promise<string>,
  Calls extends Call | Call[]
>(
  call: Promise<
    BlockCreationResponse<ApiType, Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>>
  >
) {
  const block = await call;
  if (Array.isArray(block.result)) {
    block.result.forEach((r, idx) => {
      expect(r.successful, `tx[${idx}] - ${r.error?.name}`).to.be.true;
    });
  } else {
    expect(block.result.successful, block.result.error?.name).to.be.true;
  }
}
