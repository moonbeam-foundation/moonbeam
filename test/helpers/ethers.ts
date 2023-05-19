import { DevModeContext } from "@moonwall/cli";
import { TransactionRequest } from "ethers";

import { TransactionType } from "./viem.js";
import { ALITH_ADDRESS } from "@moonwall/util";

export async function createEthersTxn<
  TOptions extends TransactionRequest & { txnType?: TransactionType }
>(context: DevModeContext, params: TOptions) {
  const isLegacy =
    params.txnType == "legacy" ||
    params.type == 0 ||
    (params.type == undefined && params.txnType == undefined); // Default to legacy
  const isEIP155 = params.type == 1;
  const isEIP1559 = params.txnType == "eip1559" || params.type == 2;
  const isEIP2930 = params.txnType == "eip2930";
  // for some reason ethers messes up the nonce
  const nonce = await context.viemClient("public").getTransactionCount({ address: ALITH_ADDRESS });
  const blob: any = { nonce, ...params };

  switch (true) {
    case isLegacy:
      blob["gasPrice"] = params.gasPrice || 10_000_000_000;
      blob["gasLimit"] = params.gasLimit || 22318;
      break;
    case isEIP155:
      blob["chainId"] = 1281; // TODO: get chainId from context
      blob["gasPrice"] = params.gasPrice || 10_000_000_000;
      blob["gasLimit"] = params.gasLimit || 22318;
      break;
    case isEIP1559:
      blob["accessList"] = params.accessList || [];
      blob["maxFeePerGas"] = params.maxFeePerGas || 10_000_000_000;
      blob["maxPriorityFeePerGas"] = params.maxPriorityFeePerGas || 0;
      blob["gasLimit"] = params.gasLimit || 22318;
      break;
    case isEIP2930:
      blob["gasPrice"] = params.gasPrice || 10_000_000_000;
      blob["gasLimit"] = params.gasLimit || 22318;
      blob["accessList"] = params.accessList || [];
      break;
    default:
      throw new Error("Unknown transaction type, update createRawEthersTxn fn");
  }

  const txn = await context.ethersSigner().populateTransaction(blob);
  const raw = await context.ethersSigner().signTransaction(txn);
  return { rawSigned: raw as `0x${string}`, request: txn };
}

function convertBigIntToNumber(obj: any) {
  for (let key in obj) {
    if (typeof obj[key] === "bigint") {
      obj[key] = Number(obj[key]);
    } else if (typeof obj[key] === "object" && obj[key] !== null) {
      convertBigIntToNumber(obj[key]);
    }
  }
  return obj;
}
