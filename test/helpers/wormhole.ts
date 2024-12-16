import { SigningKey } from "ethers";
import { encodePacked, keccak256, pad, toBytes } from "viem";

function encode(type: string, val: any) {
  if (type === "uint8") return encodePacked(["uint8"], [val]).slice(2);
  if (type === "uint16") return encodePacked(["uint16"], [val]).slice(2);
  if (type === "uint32") return encodePacked(["uint32"], [val]).slice(2);
  if (type === "uint64") return encodePacked(["uint64"], [val]).slice(2);
  if (type === "uint128") return encodePacked(["uint128"], [val]).slice(2);
  if (type === "address32") return pad(encodePacked(["address"], [`0x${val.slice(-40)}`])).slice(2);
  if (type === "uint256") return encodePacked(["uint256"], [val]).slice(2);
  if (type === "bytes32")
    return encodePacked(["bytes32"], [pad(val as `0x${string}`, { size: 32 })]).slice(2);
}

// Create a signed VAA to be sent to Wormhole bridge
export async function createSignedVAA(
  guardianSetIndex: number,
  signers: any,
  timestamp: number,
  nonce: number,
  emitterChainId: number,
  emitterAddress: `0x${string}`,
  sequence: bigint,
  consistencyLevel: number,
  payload: string
) {
  const body = [
    encode("uint32", timestamp),
    encode("uint32", nonce),
    encode("uint16", emitterChainId),
    encode("address32", emitterAddress),
    encode("uint64", sequence),
    encode("uint8", consistencyLevel),
    payload.slice(2),
  ];

  const hash = keccak256(keccak256(`0x${body.join("")}` as `0x${string}`));

  let signatures = "";
  for (const i in signers) {
    const key = new SigningKey(signers[i]);
    const signature = key.sign(toBytes(hash));

    const packSig = [
      encode("uint8", i),
      encode("bytes32", signature.r),
      encode("bytes32", signature.s),
      encode("uint8", signature.yParity),
    ];
    signatures += packSig.join("");
  }

  const vm = [
    encode("uint8", 1),
    encode("uint32", guardianSetIndex),
    encode("uint8", signers.length),

    signatures,
    body.join(""),
  ].join("");

  return vm;
}

export function genRegisterChainVAA(
  signers: any,
  tokenEmitter: string,
  guardianSet: number,
  nonce: number,
  seq: bigint,
  chain: number
) {
  const b = [
    "0x",
    "".padStart((32 - 11) * 2, "0"),
    encode("uint8", "T".charCodeAt(0)),
    encode("uint8", "o".charCodeAt(0)),
    encode("uint8", "k".charCodeAt(0)),
    encode("uint8", "e".charCodeAt(0)),
    encode("uint8", "n".charCodeAt(0)),
    encode("uint8", "B".charCodeAt(0)),
    encode("uint8", "r".charCodeAt(0)),
    encode("uint8", "i".charCodeAt(0)),
    encode("uint8", "d".charCodeAt(0)),
    encode("uint8", "g".charCodeAt(0)),
    encode("uint8", "e".charCodeAt(0)),
    encode("uint8", 1),
    encode("uint16", 0),
    encode("uint16", chain),
    encode("address32", tokenEmitter),
  ];
  const emitter: `0x${string}` = `0x${"04".padStart(64, "0")}`;
  const seconds = Math.floor(new Date().getTime() / 1000.0);

  return createSignedVAA(guardianSet, signers, seconds, nonce, 1, emitter, seq, 32, b.join(""));
}

export async function genAssetMeta(
  signers: any,
  guardianSet: number,
  nonce: number,
  seq: bigint,
  tokenAddress: string,
  tokenChain: number,
  tokenEmitter: `0x${string}`,
  decimals: number,
  symbol: string,
  name: string
) {
  const b = [
    "0x",
    encode("uint8", 2),
    encode("address32", tokenAddress),
    encode("uint16", tokenChain),
    encode("uint8", decimals),
    encode("bytes32", `0x${Buffer.from(symbol).toString("hex")}`),
    encode("bytes32", `0x${Buffer.from(name).toString("hex")}`),
  ];

  const seconds = Math.floor(new Date().getTime() / 1000.0);

  return createSignedVAA(
    guardianSet,
    signers,
    seconds,
    nonce,
    tokenChain,
    tokenEmitter,
    seq,
    32,
    b.join("")
  );
}

export function genTransferVAA(
  signers: any,
  guardianSet: number,
  nonce: number,
  seq: bigint,
  amount: number,
  tokenAddress: string,
  tokenChain: number,
  tokenEmitterChainId: number,
  tokenEmitter: `0x${string}`,
  toAddress: string,
  toChain: string,
  fee: number
) {
  const b = [
    "0x",
    encode("uint8", 1),
    encode("uint256", Math.floor(amount * 100000000)),
    encode("address32", tokenAddress),
    encode("uint16", tokenChain),
    encode("address32", toAddress),
    encode("uint16", toChain),
    encode("uint256", Math.floor(fee * 100000000)),
  ];

  const seconds = Math.floor(new Date().getTime() / 1000.0);

  return createSignedVAA(
    guardianSet,
    signers,
    seconds,
    nonce,
    tokenEmitterChainId,
    tokenEmitter,
    seq,
    32,
    b.join("")
  );
}

export function genTransferWithPayloadVAA(
  signers: any,
  guardianSet: number,
  nonce: number,
  seq: bigint,
  amount: number,
  tokenAddress: string,
  tokenChain: number,
  tokenEmitterChainId: number,
  tokenEmitter: `0x${string}`,
  toAddress: string,
  toChain: string,
  fromAddress: string,
  payload: string
) {
  const b = [
    "0x",
    encode("uint8", 3),
    encode("uint256", Math.floor(amount * 100000000)),
    encode("address32", tokenAddress),
    encode("uint16", tokenChain),
    encode("address32", toAddress),
    encode("uint16", toChain),
    encode("address32", fromAddress),
    payload.slice(2),
  ];

  const seconds = Math.floor(new Date().getTime() / 1000.0);

  return createSignedVAA(
    guardianSet,
    signers,
    seconds,
    nonce,
    tokenEmitterChainId,
    tokenEmitter,
    seq,
    32,
    b.join("")
  );
}
