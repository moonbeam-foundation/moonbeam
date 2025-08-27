import type { DevModeContext, GenericContext } from "@moonwall/cli";
import { type TransactionSerializable } from "viem";
import { ALITH_PRIVATE_KEY } from "@moonwall/util";
import { generatePrivateKey, privateKeyToAccount } from "viem/accounts";
import { fundAccount } from "../../../../helpers";

export async function createFundedAccount(context: DevModeContext) {
  const privateKey = generatePrivateKey();
  const account = privateKeyToAccount(privateKey);
  await fundAccount(account.address, 100_000_000_000_000_000_000n, context);

  return {
    account: account,
    privateKey: privateKey,
  };
}

// TODO temporary helper until the changes are merged upstream to Moonwall
export async function createViemTransaction(
  context: GenericContext,
  options
): Promise<`0x${string}`> {
  const type = !!options && !!options.txnType ? options.txnType : "eip1559";
  const privateKey = !!options && !!options.privateKey ? options.privateKey : ALITH_PRIVATE_KEY;
  const account = privateKeyToAccount(privateKey);
  const value = options?.value ? options.value : 0n;
  const to = options?.to ? options.to : "0x0000000000000000000000000000000000000000";
  const chainId = await context.viem().getChainId();
  const txnCount = await context.viem().getTransactionCount({ address: account.address });
  const gasPrice = await context.viem().getGasPrice();
  const data = options?.data ? options.data : "0x";

  let estimatedGas = 1_500_000n;
  if (!options.skipEstimation && options.gas === undefined) {
    estimatedGas = await context.viem().estimateGas({ account: account.address, to, value, data });

    // TODO find a better estimation
    estimatedGas += options?.authorizationList
      ? BigInt(options.authorizationList.length) * 500_000n
      : 0n;
  }

  const accessList = options?.accessList ? options.accessList : [];
  const authorizationList = options?.authorizationList ? options.authorizationList : [];

  const txnBlob =
    type === "eip1559"
      ? ({
          to,
          value,
          maxFeePerGas: options.maxFeePerGas !== undefined ? options.maxFeePerGas : gasPrice,
          maxPriorityFeePerGas:
            options.maxPriorityFeePerGas !== undefined ? options.maxPriorityFeePerGas : gasPrice,
          gas: options.gas !== undefined ? options.gas : estimatedGas,
          nonce: options.nonce !== undefined ? options.nonce : txnCount,
          data,
          chainId,
          type,
        } satisfies TransactionSerializable)
      : type === "legacy"
        ? ({
            to,
            value,
            gasPrice: options.gasPrice !== undefined ? options.gasPrice : gasPrice,
            gas: options.gas !== undefined ? options.gas : estimatedGas,
            nonce: options.nonce !== undefined ? options.nonce : txnCount,
            data,
          } satisfies TransactionSerializable)
        : type === "eip2930"
          ? ({
              to,
              value,
              gasPrice: options.gasPrice !== undefined ? options.gasPrice : gasPrice,
              gas: options.gas !== undefined ? options.gas : estimatedGas,
              nonce: options.nonce !== undefined ? options.nonce : txnCount,
              data,
              chainId,
              type,
            } satisfies TransactionSerializable)
          : type === "eip7702"
            ? ({
                to,
                value,
                maxFeePerGas: options.maxFeePerGas !== undefined ? options.maxFeePerGas : gasPrice,
                maxPriorityFeePerGas:
                  options.maxPriorityFeePerGas !== undefined
                    ? options.maxPriorityFeePerGas
                    : gasPrice,
                gas: options.gas !== undefined ? options.gas : estimatedGas,
                nonce: options.nonce !== undefined ? options.nonce : txnCount,
                data,
                chainId,
                authorizationList,
                type,
              } satisfies TransactionSerializable)
            : {};

  if (
    (type === "eip1559" && accessList.length > 0) ||
    (type === "eip2930" && accessList.length > 0) ||
    (type === "eip7702" && accessList.length > 0)
  ) {
    (txnBlob as any).accessList = accessList;
  }
  return await account.signTransaction(txnBlob);
}
