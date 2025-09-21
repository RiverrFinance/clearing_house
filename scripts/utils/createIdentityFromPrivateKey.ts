import { Ed25519KeyIdentity } from "@icp-sdk/core/identity";

export const createIdentityFromPrivateKey = (
  privateKeyHex: string
): Ed25519KeyIdentity => {
  let normalized = privateKeyHex.trim().toLowerCase().replace(/\s+/g, "");

  // Convert hex to bytes
  const bytePairs = normalized.match(/.{1,2}/g) ?? [];
  const privateKeyBytes = new Uint8Array(bytePairs.map((b) => parseInt(b, 16)));

  return Ed25519KeyIdentity.fromSecretKey(privateKeyBytes);
};
