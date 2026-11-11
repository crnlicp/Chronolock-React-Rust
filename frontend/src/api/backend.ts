import { Principal } from '@icp-sdk/core/principal';
import {
  useMutation,
  useQuery,
  useQueryClient,
  keepPreviousData,
} from '@tanstack/react-query';
import { useMemo } from 'react';
import { useAuth } from '../auth/AuthProvider';
import type { DerivedPublicKey } from '@icp-sdk/vetkeys';
import {
  createActor,
  type Backend,
  type Chronolock,
  type MediaRef,
} from '../bindings/backend';
import { canisterId, isLocal, rootKey } from '../lib/env';
import { unwrap } from '../lib/errors';
import {
  AES_OVERHEAD,
  SCHEME_VERSION,
  aesDecrypt,
  aesEncrypt,
  decryptPayload,
  deriveIbePublicKey,
  mediaChunkAad,
  mediaSegmentSize,
  prepareKeyRequest,
  type LockPayload,
} from '../lib/crypto';

export type { Chronolock };

export function backendId(): string {
  const id = canisterId('backend');
  if (!id) {
    throw new Error(
      'Backend canister id missing: open the app through the frontend canister or `pnpm dev`.',
    );
  }
  return id;
}

/** Backend actor for the signed-in user, or anonymous when signed out. */
export function useBackend(): Backend {
  const { identity } = useAuth();
  return useMemo(
    () => createActor(backendId(), { agentOptions: { identity, rootKey } }),
    [identity],
  );
}

export const keys = {
  all: ['backend'] as const,
  stats: () => [...keys.all, 'stats'] as const,
  config: () => [...keys.all, 'config'] as const,
  lock: (id: bigint) => [...keys.all, 'lock', id.toString()] as const,
  list: (kind: string, who: string, offset: number, limit: number, extra = '') =>
    [...keys.all, 'list', kind, who, offset, limit, extra] as const,
};

export function useStats() {
  const backend = useBackend();
  return useQuery({ queryKey: keys.stats(), queryFn: () => backend.get_stats() });
}

export function useBackendConfig() {
  const backend = useBackend();
  return useQuery({
    queryKey: keys.config(),
    queryFn: () => backend.get_config(),
    staleTime: 5 * 60_000,
  });
}

export function useChronolock(id: bigint | undefined) {
  const backend = useBackend();
  return useQuery({
    queryKey: keys.lock(id ?? -1n),
    queryFn: () => backend.get_chronolock(id!),
    enabled: id !== undefined,
  });
}

export type ListKind = 'all' | 'owned' | 'received' | 'unlockable';

export function useChronolockList(
  kind: ListKind,
  offset: number,
  limit: number,
  who?: string,
) {
  const backend = useBackend();
  const page = { offset: BigInt(offset), limit: BigInt(limit) };
  return useQuery({
    queryKey: keys.list(kind, who ?? '', offset, limit),
    enabled: kind === 'all' || !!who,
    placeholderData: keepPreviousData,
    queryFn: () => {
      switch (kind) {
        case 'all':
          return backend.list_chronolocks(page);
        case 'owned':
          return backend.list_owned_chronolocks(Principal.fromText(who!), page);
        case 'received':
          return backend.list_received_chronolocks(Principal.fromText(who!), false, page);
        case 'unlockable':
          return backend.list_received_chronolocks(Principal.fromText(who!), true, page);
      }
    },
  });
}

export function useBurnChronolock() {
  const backend = useBackend();
  const qc = useQueryClient();
  return useMutation({
    mutationFn: async (id: bigint) => unwrap(await backend.burn_chronolock(id)),
    onSuccess: () => qc.invalidateQueries({ queryKey: keys.all }),
  });
}

// ---------------------------------------------------------------------------
// Encryption key
// ---------------------------------------------------------------------------

export type Network = 'mainnet' | 'local';

/**
 * The backend's IBE public key, derived offline from the network's vetKD
 * master key and the canister id. The canister's own answer is never trusted.
 */
export function getIbePublicKey(
  canister: string,
  keyName: string,
  network: Network,
): DerivedPublicKey {
  return deriveIbePublicKey(canister, keyName, network);
}

export function currentNetwork(): Network {
  return isLocal ? 'local' : 'mainnet';
}

/** IBE public key for the current page's backend. */
export async function ibePublicKeyFor(backend: Backend): Promise<DerivedPublicKey> {
  const cfg = await backend.get_config();
  return getIbePublicKey(backendId(), cfg.vetkd_key_name, currentNetwork());
}

// ---------------------------------------------------------------------------
// Media
// ---------------------------------------------------------------------------

/**
 * Encrypts and uploads `plain` chunk by chunk. Each chunk is bound to the
 * upload id, its index and the chunk count (AES-GCM associated data).
 */
export async function uploadMedia(
  backend: Backend,
  key: CryptoKey,
  plain: Uint8Array,
  maxChunkBytes: number,
  onProgress?: (done: number, total: number) => void,
): Promise<bigint> {
  const segment = mediaSegmentSize(maxChunkBytes);
  const count = Math.max(1, Math.ceil(plain.length / segment));
  const size = BigInt(plain.length + count * AES_OVERHEAD);
  const mediaId = unwrap(await backend.start_media_upload({ size, chunk_count: count }));
  try {
    for (let i = 0; i < count; i++) {
      const part = plain.subarray(i * segment, Math.min((i + 1) * segment, plain.length));
      const chunk = await aesEncrypt(key, part, mediaChunkAad(mediaId, i, count));
      unwrap(await backend.upload_media_chunk(mediaId, i, chunk));
      onProgress?.(i + 1, count);
    }
    unwrap(await backend.finish_media_upload(mediaId));
    return mediaId;
  } catch (err) {
    await backend.cancel_media_upload(mediaId).catch(() => undefined);
    throw err;
  }
}

export async function downloadMedia(
  backend: Backend,
  key: CryptoKey,
  media: MediaRef,
  onProgress?: (done: number, total: number) => void,
): Promise<Uint8Array> {
  const parts: Uint8Array[] = [];
  for (let i = 0; i < media.chunk_count; i++) {
    const chunk = unwrap(await backend.get_media_chunk(media.media_id, i));
    parts.push(await aesDecrypt(key, chunk, mediaChunkAad(media.media_id, i, media.chunk_count)));
    onProgress?.(i + 1, media.chunk_count);
  }
  const out = new Uint8Array(parts.reduce((n, c) => n + c.length, 0));
  let pos = 0;
  for (const c of parts) {
    out.set(c, pos);
    pos += c.length;
  }
  return out;
}

// ---------------------------------------------------------------------------
// Decryption
// ---------------------------------------------------------------------------

export interface DecryptedLock extends LockPayload {
  mediaUrl?: string;
}

export async function decryptChronolock(
  backend: Backend,
  lock: Chronolock,
  principal: string,
  ibePublicKey: DerivedPublicKey,
  onStep?: (step: string) => void,
): Promise<DecryptedLock> {
  if (lock.scheme !== SCHEME_VERSION) {
    throw new Error(`Unsupported encryption scheme v${lock.scheme}.`);
  }
  const req = prepareKeyRequest(lock.user_keys, principal, lock.unlock_time);
  if (!req) throw new Error('This chronolock was not shared with you.');

  onStep?.('Requesting time-lock key…');
  const encryptedKey = unwrap(
    req.isPublic
      ? await backend.public_time_key(lock.unlock_time, req.transportPublicKey)
      : await backend.user_time_key(lock.unlock_time, req.transportPublicKey),
  );
  const aesKey = await req.unwrap(encryptedKey, ibePublicKey);

  onStep?.('Decrypting content…');
  const payload = await decryptPayload(aesKey, lock.encrypted_metadata);

  if (!lock.media) return payload;
  const media = await downloadMedia(backend, aesKey, lock.media, (done, total) =>
    onStep?.(`Decrypting media (${done}/${total})…`),
  );
  const blob = new Blob([media as BlobPart], {
    type: payload.fileType || 'application/octet-stream',
  });
  return { ...payload, mediaUrl: URL.createObjectURL(blob) };
}
