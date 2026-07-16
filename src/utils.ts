// Crypto utils
import { Buffer } from "buffer";
import bs58 from "bs58";
import { sha256 } from "@noble/hashes/sha2.js";
import { PRIVKEY_BYTE_LENGTH, SECRET_KEY } from "./constants";

type Bytes = Uint8Array | Buffer;

export { sha256 };

export function dsha256(bytes: Bytes) {
  return sha256(sha256(bytes));
}

export function hexToBytes(str: string) {
  return new Uint8Array(Buffer.from(str, "hex"));
}

export function bytesToHex(bytes: Bytes) {
  return Buffer.from(bytes).toString("hex");
}

export function numToBytes(number: number, numBytes = 8, littleEndian = true) {
  const bytes = [];
  for (let i = 0; i < numBytes; i++) {
    bytes.push((number / 2 ** (8 * i)) & 0xff);
  }
  return littleEndian ? bytes : bytes.reverse();
}

function bytesEqual(a: Bytes, b: Bytes) {
  return a.length === b.length && a.every((val, i) => val === b[i]);
}

export function parseWIF(wif: string) {
  const bytes = bs58.decode(wif);
  if (bytes.byteLength !== PRIVKEY_BYTE_LENGTH)
    throw new Error(
      `Private key should have length ${PRIVKEY_BYTE_LENGTH} but has ${bytes.byteLength} length`,
    );
  if (bytes[0] !== SECRET_KEY) {
    throw new Error(
      `Invalid key, this key belongs to another coin or is corrupted`,
    );
  }
  const hash = dsha256(bytes.slice(0, PRIVKEY_BYTE_LENGTH - 4));
  const checksum = bytes.slice(PRIVKEY_BYTE_LENGTH - 4);
  const computedChecksum = hash.slice(0, 4);

  if (!bytesEqual(checksum, computedChecksum))
    throw new Error("Invalid checksum");
  return bytes.slice(1, 33);
}
