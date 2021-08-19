import { ApiPromise } from "@polkadot/api";

export type Extrinsic = ReturnType<ApiPromise["tx"]>;
