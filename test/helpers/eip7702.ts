import type { PrivateKeyAccount } from "viem";
import { keccak256, toHex, concat, type Hex, parseAbi, toRlp } from "viem";

/**
 * EIP-7702 Authorization structure compatible with ethers.js
 */
export interface EIP7702Authorization {
  chainId: bigint;
  address: Hex; // The contract address being delegated to
  nonce: bigint; // The nonce of the authorizing account
  signature: {
    v: number;
    r: Hex;
    s: Hex;
  };
}

/**
 * Creates an EIP-7702 authorization hash according to the specification
 * @param chainId - Chain ID (0 for cross-chain compatibility or specific chain ID)
 * @param contractAddress - The contract address being delegated to
 * @param nonce - The nonce of the authorizing account
 * @returns The authorization hash to be signed
 */
function createEIP7702AuthorizationHash(chainId: bigint, contractAddress: Hex, nonce: bigint): Hex {
  const MAGIC = "0x05" as Hex;
  const rlpData = toRlp([toHex(chainId), contractAddress, toHex(nonce)]);
  return keccak256(concat([MAGIC, rlpData]));
}

/**
 * Creates an EIP-7702 authorization object by signing the authorization hash
 * @param account - The account that will authorize the delegation
 * @param chainId - Chain ID (0 for cross-chain compatibility or specific chain ID)
 * @param nonce - The current nonce of the authorizing account
 * @param contractAddress - The contract address to delegate to
 * @returns A signed EIP-7702 authorization object
 */
export async function createEIP7702Authorization(
  account: PrivateKeyAccount,
  chainId: bigint,
  nonce: bigint,
  contractAddress: Hex
): Promise<EIP7702Authorization> {
  const authorizationHash = createEIP7702AuthorizationHash(chainId, contractAddress, nonce);

  const signature = await account.signMessage({
    message: { raw: authorizationHash },
  });

  // Parse signature components
  const r = signature.slice(0, 66) as Hex;
  const s = ("0x" + signature.slice(66, 130)) as Hex;
  const recoveryId = parseInt(signature.slice(130, 132), 16);

  // Convert recovery ID to v value (0 or 1 for EIP-7702)
  let v = BigInt(recoveryId);
  if (v >= 27n) {
    v = v - 27n;
  }

  return {
    chainId,
    address: contractAddress,
    nonce,
    signature: {
      v: Number(v),
      r,
      s,
    },
  };
}
