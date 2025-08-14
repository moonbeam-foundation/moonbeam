import {
  type DevModeContext,
  customDevRpcRequest,
  deployCreateCompiledContract,
} from "@moonwall/cli";
import { alith, createEthersTransaction } from "@moonwall/util";
import { type Abi, encodeFunctionData } from "viem";

export async function createContracts(context: DevModeContext) {
  let nonce = await context.viem().getTransactionCount({ address: alith.address as `0x${string}` });
  const { contractAddress: callee, abi: abiCallee } = await deployCreateCompiledContract(
    context,
    "TraceCallee",
    { nonce: nonce++ }
  );

  const { contractAddress: caller, abi: abiCaller } = await deployCreateCompiledContract(
    context,
    "TraceCaller",
    { nonce: nonce++ }
  );
  await context.createBlock();

  return {
    abiCallee,
    abiCaller,
    calleeAddr: callee,
    callerAddr: caller,
    nonce: nonce,
  };
}

export async function nestedCall(
  context: DevModeContext,
  callerAddr: string,
  calleeAddr: string,
  abiCaller: Abi,
  nonce: number
) {
  const callTx = await createEthersTransaction(context, {
    to: callerAddr,
    data: encodeFunctionData({
      abi: abiCaller,
      functionName: "someAction",
      args: [calleeAddr, 6],
    }),
    nonce: nonce,
    gasLimit: "0x100000",
    value: "0x00",
  });
  return await customDevRpcRequest("eth_sendRawTransaction", [callTx]);
}

export async function nestedSingle(context: DevModeContext) {
  const contracts = await createContracts(context);
  return await nestedCall(
    context,
    contracts.callerAddr,
    contracts.calleeAddr,
    contracts.abiCaller,
    contracts.nonce
  );
}
