import { DefinitionRpc, DefinitionRpcSub } from '@polkadot/types/types';

declare const rpcDefinitions: Record<string, Record<string, DefinitionRpc | DefinitionRpcSub>>;

export { rpcDefinitions };
