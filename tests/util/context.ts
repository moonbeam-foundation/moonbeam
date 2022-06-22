import { ApiPromise } from "@polkadot/api";
import { ApiTypes, SubmittableExtrinsic } from "@polkadot/api/types";
import { ethers } from "ethers";

import { EnhancedWeb3 } from "./providers";
import { ExtrinsicCreation } from "./substrate-rpc";

export interface BlockCreation {
  parentHash?: string;
  finalize?: boolean;
}

export interface BlockCreationResponse<
  ApiType extends ApiTypes,
  Call extends SubmittableExtrinsic<ApiType> | string | (SubmittableExtrinsic<ApiType> | string)[]
> {
  block: {
    duration: number;
    hash: string;
  };
  result: Call extends (string | SubmittableExtrinsic<ApiType>)[]
    ? ExtrinsicCreation[]
    : ExtrinsicCreation;
}

export type EthTransactionType = "Legacy" | "EIP2930" | "EIP1559";

export interface TestContext {
  createBlock<
    ApiType extends ApiTypes,
    Call extends
      | SubmittableExtrinsic<ApiType>
      | Promise<SubmittableExtrinsic<ApiType>>
      | string
      | Promise<string>,
    Calls extends Call | Call[]
  >(
    transactions?: Calls,
    options?: BlockCreation
  ): Promise<
    BlockCreationResponse<ApiType, Calls extends Call[] ? Awaited<Call>[] : Awaited<Call>>
  >;
  createWeb3: (protocol?: "ws" | "http") => Promise<EnhancedWeb3>;
  createEthers: () => Promise<ethers.providers.JsonRpcProvider>;

  // We also provided singleton providers for simplicity
  web3: EnhancedWeb3;
  ethers: ethers.providers.JsonRpcProvider;
  polkadotApi: ApiPromise;
}
