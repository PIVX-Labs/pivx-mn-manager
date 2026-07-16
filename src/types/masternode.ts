import { Buffer } from "buffer";
import { dsha256, hexToBytes, numToBytes, parseWIF } from "../utils";
import * as nobleSecp256k1 from "@noble/secp256k1";

const DEFAULT_NODE = "https://rpc.duddino.com/mainnet";

export type Masternode = {
  name: string;
  ipAddress: string;
  privateKey: string;
  collateralTxId: string;
  outId: number;
};

const MN_STATUSES = ["ENABLED", "MISSING_COLLATERAL", "MISSING", "PRE_ENABLED"];
export type MasternodeStatuses = (typeof MN_STATUSES)[number];

type ListMasternodeResponse = {
  status: MasternodeStatuses;
};

export async function getMasternodeStatus(
  masternode: Masternode,
): Promise<MasternodeStatuses> {
  if (!masternode.collateralTxId) return "MISSING_COLLATERAL";
  const response = await fetch(
    `${DEFAULT_NODE}/listmasternodes?params=${masternode.collateralTxId}`,
  );
  const isValidResponse = async (
    resp: typeof response,
  ): Promise<ListMasternodeResponse | null> => {
    if (!resp.ok) return null;
    try {
      const obj = await resp.json();
      if (obj && MN_STATUSES.includes(obj.status)) {
        return obj;
      } else {
        console.warn(`Warning: Invalid response return from server: ${obj}`);
        return null;
      }
    } catch {
      return null;
    }
  };
  const mnResponse = await isValidResponse(response);
  if (!mnResponse) return "MISSING";
  return mnResponse.status;
}

export async function masterdeVote(
  masternode: Masternode,
  hash: string,
  voteCode: 1 | 2,
): Promise<boolean> {
  try {
    const sigTime = Math.round(Date.now() / 1000);
    const msg = new Uint8Array([
      ...hexToBytes(masternode.collateralTxId).reverse(),
      ...numToBytes(masternode.outId, 4),
      ...[0, 255, 255, 255, 255],
      ...hexToBytes(hash).reverse(),
      ...numToBytes(voteCode, 4),
      ...numToBytes(sigTime, 8),
    ]);
    const signature = await nobleSecp256k1.signAsync(
      dsha256(msg),
      parseWIF(masternode.privateKey),
      { format: "recovered", prehash: false },
    );
    const signedMessage = Buffer.from([
      signature[0] + 27,
      ...signature.slice(1),
    ]).toString("base64");

    const encodedSignature = encodeURI(signedMessage).replaceAll("+", "%2b");
    const response = await fetch(
      `${DEFAULT_NODE}/mnbudgetrawvote?params=${masternode.collateralTxId},${masternode.outId},${hash},${voteCode === 1 ? "yes" : "no"},${sigTime},${encodedSignature}`,
    );
    return response.ok;
  } catch {
    return false;
  }
}
