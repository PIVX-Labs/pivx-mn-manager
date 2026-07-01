const DEFAULT_NODE = "https://rpc.duddino.com/mainnet";

export type Masternode = {
  name: string;
  ipAddress: string;
  privateKey: string;
  collateralTxId: string;
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
