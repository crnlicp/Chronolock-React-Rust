/**
 * Chronolock encryption, scheme v2 (see docs/ENCRYPTION_AND_DECRYPTION.md).
 *
 *  - Each lock gets a random AES-256-GCM key. The metadata (including the
 *    title) and every media chunk are encrypted with it in the browser.
 *  - The AES key is IBE-encrypted (vetKeys) to an identity binding the unlock
 *    time and, for private locks, the recipient principal.
 *  - After the unlock time the backend derives the matching vetKey for the
 *    caller, delivered under a one-time transport key.
 */
import { Principal } from '@icp-sdk/core/principal';
import {
  DerivedPublicKey,
  EncryptedVetKey,
  IbeCiphertext,
  IbeIdentity,
  IbeSeed,
  MasterPublicKey,
  MasterPublicKeyId,
  TransportSecretKey,
} from '@icp-sdk/vetkeys';
import type { UserKey } from '../bindings/backend';

export const SCHEME_VERSION = 2;
/** vetKD context (domain separator). Must match backend/src/vetkd.rs. */
export const VETKD_CONTEXT = new TextEncoder().encode('chronolock/v2');
export const PUBLIC_RECIPIENT = 'public';

const IV_LEN = 12;
const TAG_LEN = 16;
/** Bytes AES-GCM adds to every encrypted message. */
export const AES_OVERHEAD = IV_LEN + TAG_LEN;
const META_AAD = new TextEncoder().encode('chronolock/v2/meta');
const MEDIA_AAD_PREFIX = new TextEncoder().encode('chronolock/v2/media');

// ---------------------------------------------------------------------------
// Encoding helpers
// ---------------------------------------------------------------------------

export function toBase64(bytes: Uint8Array): string {
  let s = '';
  for (let i = 0; i < bytes.length; i += 0x8000) {
    s += String.fromCharCode(...bytes.subarray(i, i + 0x8000));
  }
  return btoa(s);
}

export function fromBase64(b64: string): Uint8Array {
  return Uint8Array.from(atob(b64), (c) => c.charCodeAt(0));
}

function concat(...parts: Uint8Array[]): Uint8Array {
  const out = new Uint8Array(parts.reduce((n, p) => n + p.length, 0));
  let pos = 0;
  for (const p of parts) {
    out.set(p, pos);
    pos += p.length;
  }
  return out;
}

function u64be(n: bigint): Uint8Array {
  const b = new Uint8Array(8);
  new DataView(b.buffer).setBigUint64(0, n);
  return b;
}

function u32be(n: number): Uint8Array {
  const b = new Uint8Array(4);
  new DataView(b.buffer).setUint32(0, n);
  return b;
}

// ---------------------------------------------------------------------------
// AES-GCM
// ---------------------------------------------------------------------------

export function generateAesKey(): Promise<CryptoKey> {
  return crypto.subtle.generateKey({ name: 'AES-GCM', length: 256 }, true, [
    'encrypt',
    'decrypt',
  ]);
}

/** Returns iv || ciphertext || tag. */
export async function aesEncrypt(
  key: CryptoKey,
  data: Uint8Array,
  aad: Uint8Array,
): Promise<Uint8Array> {
  const iv = crypto.getRandomValues(new Uint8Array(IV_LEN));
  const ct = await crypto.subtle.encrypt(
    { name: 'AES-GCM', iv, additionalData: aad as BufferSource },
    key,
    data as BufferSource,
  );
  return concat(iv, new Uint8Array(ct));
}

export async function aesDecrypt(
  key: CryptoKey,
  data: Uint8Array,
  aad: Uint8Array,
): Promise<Uint8Array> {
  if (data.length < AES_OVERHEAD) throw new Error('Encrypted data is too short');
  const pt = await crypto.subtle.decrypt(
    { name: 'AES-GCM', iv: data.slice(0, IV_LEN), additionalData: aad as BufferSource },
    key,
    data.slice(IV_LEN),
  );
  return new Uint8Array(pt);
}

// ---------------------------------------------------------------------------
// vetKeys IBE
// ---------------------------------------------------------------------------

/**
 * IBE identity bytes. Must match `identity()` in backend/src/vetkd.rs:
 * "CL" || version || kind || unlock_time (u64 BE) [|| len || principal]
 */
export function ibeIdentity(unlockTime: bigint, recipient?: string): Uint8Array {
  const isPublic = !recipient || recipient === PUBLIC_RECIPIENT;
  const head = new Uint8Array([0x43, 0x4c, SCHEME_VERSION, isPublic ? 0 : 1]);
  if (isPublic) return concat(head, u64be(unlockTime));
  const p = Principal.fromText(recipient).toUint8Array();
  return concat(head, u64be(unlockTime), new Uint8Array([p.length]), p);
}

/**
 * The canister's IBE public key, derived offline from the network's master
 * key so a tampered canister response cannot redirect encryption.
 */
export function deriveIbePublicKey(
  canisterId: string,
  keyName: string,
  network: 'mainnet' | 'local',
): DerivedPublicKey {
  const master =
    network === 'mainnet'
      ? MasterPublicKey.productionKey(keyName as MasterPublicKeyId)
      : MasterPublicKey.pocketicKey(keyName as never);
  return master
    .deriveCanisterKey(Principal.fromText(canisterId).toUint8Array())
    .deriveSubKey(VETKD_CONTEXT);
}

/** Wraps the AES key for each recipient, or for everyone if there are none. */
export async function wrapKey(
  aesKey: CryptoKey,
  ibePublicKey: DerivedPublicKey,
  recipients: string[],
  unlockTime: bigint,
): Promise<UserKey[]> {
  const raw = new Uint8Array(await crypto.subtle.exportKey('raw', aesKey));
  const users = recipients.length > 0 ? recipients : [PUBLIC_RECIPIENT];
  return users.map((user) => ({
    user,
    key: toBase64(
      IbeCiphertext.encrypt(
        ibePublicKey,
        IbeIdentity.fromBytes(ibeIdentity(unlockTime, user)),
        raw,
        IbeSeed.random(),
      ).serialize(),
    ),
  }));
}

export interface KeyRequest {
  isPublic: boolean;
  /** Fresh one-time transport public key to send to the canister. */
  transportPublicKey: Uint8Array;
  /** Turns the canister's encrypted vetKey into the lock's AES key. */
  unwrap: (encryptedVetKey: Uint8Array, ibePublicKey: DerivedPublicKey) => Promise<CryptoKey>;
}

/** Picks the caller's key entry (or the public one) and prepares a vetKey request. */
export function prepareKeyRequest(
  userKeys: UserKey[],
  principal: string,
  unlockTime: bigint,
): KeyRequest | null {
  const entry =
    userKeys.find((k) => k.user === principal) ??
    userKeys.find((k) => k.user === PUBLIC_RECIPIENT);
  if (!entry) return null;
  const tsk = TransportSecretKey.random();
  const identity = ibeIdentity(unlockTime, entry.user);
  return {
    isPublic: entry.user === PUBLIC_RECIPIENT,
    transportPublicKey: tsk.publicKeyBytes(),
    unwrap: async (encryptedVetKey, ibePublicKey) => {
      const vetKey = EncryptedVetKey.deserialize(encryptedVetKey).decryptAndVerify(
        tsk,
        ibePublicKey,
        identity,
      );
      const raw = IbeCiphertext.deserialize(fromBase64(entry.key)).decrypt(vetKey);
      return crypto.subtle.importKey('raw', raw as BufferSource, { name: 'AES-GCM' }, false, [
        'decrypt',
      ]);
    },
  };
}

// ---------------------------------------------------------------------------
// Payload and media
// ---------------------------------------------------------------------------

/** Decrypted content of a chronolock. */
export interface LockPayload {
  title: string;
  name?: string;
  description?: string;
  fileName?: string;
  fileType?: string;
  /** Plaintext media size in bytes. */
  mediaSize?: number;
  attributes?: { key: string; value: string }[];
}

export async function encryptPayload(key: CryptoKey, payload: LockPayload): Promise<string> {
  const data = new TextEncoder().encode(JSON.stringify(payload));
  return toBase64(await aesEncrypt(key, data, META_AAD));
}

export async function decryptPayload(key: CryptoKey, b64: string): Promise<LockPayload> {
  const data = await aesDecrypt(key, fromBase64(b64), META_AAD);
  return JSON.parse(new TextDecoder().decode(data));
}

/** Binds a media chunk to its upload, position and total, so chunks can't be swapped. */
export function mediaChunkAad(mediaId: bigint, index: number, count: number): Uint8Array {
  return concat(MEDIA_AAD_PREFIX, u64be(mediaId), u32be(index), u32be(count));
}

/** Plaintext bytes per media chunk so each encrypted chunk fits `maxChunkBytes`. */
export function mediaSegmentSize(maxChunkBytes: number): number {
  return maxChunkBytes - AES_OVERHEAD;
}
