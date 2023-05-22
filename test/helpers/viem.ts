// // TODO: Refactor these into moonwall util once they have matured

// import {
//   ContractDeploymentOptions,
//   DevModeContext,
//   PublicViem,
//   deployViemContract,
// } from "@moonwall/cli";
// import { ALITH_ADDRESS, ALITH_PRIVATE_KEY } from "@moonwall/util";
// import {
//   BlockTag,
//   DeployContractParameters,
//   PublicClient,
//   TransactionSerializable,
//   WalletClient,
//   encodeDeployData,
//   getContract,
//   keccak256,
//   numberToHex,
//   toRlp,
// } from "viem";
// import { privateKeyToAccount } from "viem/accounts";
// import { getCompiled } from "./contracts.js";
// import * as RLP from "rlp";
// import { Abi } from "abitype";

// export type InputAmountFormats = number | bigint | string | `0x${string}`;

// export type DeepPartial<T> = {
//   [P in keyof T]?: T[P] extends (infer U)[]
//     ? DeepPartial<U>[]
//     : T[P] extends ReadonlyArray<infer U>
//     ? ReadonlyArray<DeepPartial<U>>
//     : DeepPartial<T[P]>;
// };

// export type TransferOptions =
//   | (Omit<TransactionSerializable, "to" | "value"> & {
//       privateKey?: `0x${string}`;
//     })
//   | undefined;

// export type TransactionOptions =
//   | TransactionSerializable & {
//       privateKey?: `0x${string}`;
//     };

// export const TransactionTypes = ["eip1559", "eip2930", "legacy"] as const;
// export type TransactionType = (typeof TransactionTypes)[number];

// /**
//  * createRawTransfer function creates and signs a transfer, as a hex string, that can be submitted to the network via public client."
//  *
//  * @export
//  * @template TOptions - Optional parameters of Viem's TransferOptions
//  * @param {DevModeContext} context - the DevModeContext instance
//  * @param {`0x${string}`} to - the destination address of the transfer
//  * @param {InputAmountFormats} value - the amount to transfer. It accepts different formats including number, bigint, string or hexadecimal strings
//  * @param {TOptions} [options] - (optional) additional transaction options
//  * @returns {Promise<string>} - the signed raw transaction in hexadecimal string format
//  */
// export async function createRawTransfer<TOptions extends TransferOptions>(
//   context: DevModeContext,
//   to: `0x${string}`,
//   value: InputAmountFormats,
//   options?: TOptions
// ): Promise<string> {
//   const transferAmount = typeof value === "bigint" ? value : BigInt(value);
//   return await createRawTransaction(context, { ...options, to, value: transferAmount });
// }

// /**
//  * createRawTransaction function creates and signs a raw transaction, as a hex string, that can be submitted to the network via public client."
//  *
//  * @export
//  * @template TOptions - Optional parameters of Viem's TransactionOptions
//  * @param {DevModeContext} context - the DevModeContext instance
//  * @param {TOptions} options - transaction options including type, privateKey, value, to, chainId, gasPrice, estimatedGas, accessList, data
//  * @returns {Promise<string>} - the signed raw transaction in hexadecimal string format
//  */
// export async function createRawTransaction<TOptions extends DeepPartial<TransactionOptions>>(
//   context: DevModeContext,
//   options: TOptions
// ): Promise<`0x${string}`> {
//   const type = !!options && !!options.type ? options.type : "eip1559";
//   const privateKey = !!options && !!options.privateKey ? options.privateKey : ALITH_PRIVATE_KEY;
//   const account = privateKeyToAccount(privateKey);
//   const value = options && options.value ? options.value : 0n;
//   const to = options && options.to ? options.to : "0x0000000000000000000000000000000000000000";
//   const chainId = await context.viemClient("public").getChainId();
//   const txnCount = await context
//     .viemClient("public")
//     .getTransactionCount({ address: account.address });
//   const gasPrice = await context.viemClient("public").getGasPrice();
//   const estimatedGas = await context
//     .viemClient("public")
//     .estimateGas({ account: account.address, to, value });
//   const accessList = options && options.accessList ? options.accessList : [];
//   const data = options && options.data ? options.data : "0x";

//   const txnBlob: TransactionSerializable =
//     type === "eip1559"
//       ? {
//           to,
//           value,
//           maxFeePerGas: options && options.maxFeePerGas ? options.maxFeePerGas : gasPrice,
//           maxPriorityFeePerGas:
//             options && options.maxPriorityFeePerGas ? options.maxPriorityFeePerGas : gasPrice,
//           gas: options && options.gas ? options.gas : estimatedGas,
//           nonce: options && options.nonce ? options.nonce : txnCount,
//           data,
//           chainId,
//           type,
//         }
//       : type === "legacy"
//       ? {
//           to,
//           value,
//           gasPrice: options && options.gasPrice ? options.gasPrice : gasPrice,
//           gas: options && options.gas ? options.gas : estimatedGas,
//           nonce: options && options.nonce ? options.nonce : txnCount,
//           data,
//         }
//       : type === "eip2930"
//       ? {
//           to,
//           value,
//           gasPrice: options && options.gasPrice ? options.gasPrice : gasPrice,
//           gas: options && options.gas ? options.gas : estimatedGas,
//           nonce: options && options.nonce ? options.nonce : txnCount,
//           data,
//           chainId,
//           type,
//         }
//       : {};

//   if (type !== "legacy" && accessList.length > 0) {
//     // @ts-expect-error
//     txnBlob["accessList"] = accessList;
//   }

//   return await account.signTransaction(txnBlob);
// }

// /**
//  * checkBalance function checks the balance of a given account.
//  *
//  * @export
//  * @param {DevModeContext} context - the DevModeContext instance
//  * @param {`0x${string}`} [account=ALITH_ADDRESS] - the account address whose balance is to be checked. If no account is provided, it defaults to ALITH_ADDRESS
//  * @returns {Promise<bigint>} - returns a Promise that resolves to the account's balance as a BigInt
//  */
// export async function checkBalance(
//   context: DevModeContext,
//   account: `0x${string}` = ALITH_ADDRESS,
//   block: BlockTag | bigint = "latest"
// ): Promise<bigint> {
//   return typeof block == "string"
//     ? await context.viemClient("public").getBalance({ address: account, blockTag: block })
//     : typeof block == "bigint"
//     ? await context.viemClient("public").getBalance({ address: account, blockNumber: block })
//     : await context.viemClient("public").getBalance({ address: account });
// }

// /**
//  * Sends a raw signed transaction on to RPC node for execution.
//  *
//  * @async
//  * @function
//  * @param {DevModeContext} context - The DevModeContext for the Ethereum client interaction.
//  * @param {`0x${string}`} rawTx - The signed and serialized hexadecimal transaction string.
//  * @returns {Promise<any>} A Promise resolving when the transaction is sent or rejecting with an error.
//  */
// export async function sendRawTransaction(
//   context: DevModeContext,
//   rawTx: `0x${string}`
// ): Promise<any> {
//   return await context
//     .viemClient("public")
//     .request({ method: "eth_sendRawTransaction", params: [rawTx] });
// }

// // export async function callRawTransaction(
// //   context: DevModeContext,
// //   txnArgs: {}
// // ): Promise<any> {
// //   return await context
// //     .viemClient("public")
// //     .request({ method: "eth_call", params: [txnArgs] });
// // }

// export async function deployCreateCompiledContract<TOptions extends ContractDeploymentOptions>(
//   context: DevModeContext,
//   contractName: string,
//   options?: TOptions
// ) {
//   const contractCompiled = getCompiled(contractName);

//   const { privateKey = ALITH_PRIVATE_KEY, args = [], ...rest } = options || {};

//   const blob: ContractDeploymentOptions = {
//     ...rest,
//     privateKey,
//     args,
//   };

//   const { contractAddress, logs, status, hash } = await deployViemContract(
//     context,
//     contractCompiled.contract.abi as Abi,
//     contractCompiled.byteCode as `0x${string}`,
//     blob
//   );

//   const contract = getContract({
//     address: contractAddress!,
//     abi: contractCompiled.contract.abi,
//     publicClient: context.viemClient("public") as PublicClient,
//     walletClient: context.viemClient("wallet") as WalletClient,
//   });

//   return {
//     contractAddress: contractAddress as `0x${string}`,
//     contract,
//     logs,
//     hash,
//     status,
//     abi: contractCompiled.contract.abi,
//     bytecode: contractCompiled.byteCode,
//   };
// }

// export async function prepareToDeployCompiledContract<TOptions extends ContractDeploymentOptions>(
//   context: DevModeContext,
//   contractName: string,
//   options?: TOptions
// ) {
//   const compiled = getCompiled("MultiplyBy7");
//   const callData = encodeDeployData({
//     abi: compiled.contract.abi,
//     bytecode: compiled.byteCode,
//     args: [],
//   }) as `0x${string}`;

//   const nonce = await context.viemClient("public").getTransactionCount({ address: ALITH_ADDRESS });

//   await context.viemClient("wallet").sendTransaction({ data: callData, nonce });

//   const contractAddress = ("0x" +
//     keccak256(RLP.encode([ALITH_ADDRESS, nonce]))
//       .slice(12)
//       .substring(14)) as `0x${string}`;

//   return { contractAddress, callData, abi: compiled.contract.abi, bytecode: compiled.byteCode };
// }
