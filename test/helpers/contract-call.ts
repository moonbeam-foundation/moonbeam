import { DevModeContext } from "@moonwall/cli";

export class ContractCall {
  context: DevModeContext;
  privateKey?: `0x${string}`;
  gas?: bigint;
  rawTxOnly?: boolean = true;

  constructor(context: DevModeContext) {
    this.context = context;
  }

  withPrivateKey(privateKey: `0x${string}`) {
    this.privateKey = privateKey;
    return this;
  }

  withGas(gas: bigint) {
    this.gas = gas;
    return this;
  }

  withRawTxOnly(rawTxOnly: boolean) {
    if (rawTxOnly == false) {
      this.rawTxOnly = undefined;
    }
    return this;
  }

  async callExtrinsic(functionName: string, args: (number | bigint | `0x${string}`)[]) {
    return await this.callRpc(functionName, args, true);
  }

  async callQuery(functionName: string, args: (number | bigint | `0x${string}`)[]) {
    return await this.callRpc(functionName, args, false);
  }

  private async callRpc(
    functionName: string,
    args: (number | bigint | `0x${string}`)[],
    isExtrinsic: boolean
  ) {
    const params = {
      precompileName: "ConvictionVoting",
      functionName,
      args,
      privateKey: this.privateKey,
      rawTxOnly: this.rawTxOnly,
      gas: this.gas,
    };
    if (!isExtrinsic) {
      return await this.context.readPrecompile!(params);
    }
    const rawTx = await this.context.writePrecompile!(params);
    const block = await this.context.createBlock(rawTx);
    return block;
  }
}
