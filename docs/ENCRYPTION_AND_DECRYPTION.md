# Chronolock encryption (scheme v2)

Content is encrypted in the browser. The backend never sees plaintext and never
holds a key that decrypts it; it only releases a time-locked key to the right
caller once the unlock time has passed. This follows the vetKeys IBE pattern
from the ICP skills, with the identity bound to an unlock time.

## Pieces

| Piece | Where | What |
|---|---|---|
| Lock key | browser | random AES-256-GCM key, one per chronolock |
| IBE public key | browser | derived **offline** from the network's vetKD master key + backend canister id + context `chronolock/v2` (`MasterPublicKey.productionKey('key_1')` on mainnet, the PocketIC test key locally) |
| Time-lock key | backend | vetKey for an identity, derived with `vetkd_derive_key` only after the unlock time, encrypted to a one-time transport key |

Code: `frontend/src/lib/crypto.ts`, `frontend/src/api/backend.ts`, `backend/src/vetkd.rs`.

## Identities

The lock key is IBE-encrypted to one identity per recipient, or to a single
public identity when there are no recipients:

```text
identity = "CL" || 0x02 || kind || unlock_time (u64, big endian) [|| len(principal) || principal bytes]
kind 0 = public:     anyone signed in may open after unlock_time
kind 1 = recipient:  only that principal may open after unlock_time
```

The backend builds the identity itself from the caller (`msg_caller`) and the
requested unlock time, so nobody can ask for someone else's key. Both sides are
covered by matching tests (`identity_layout` in Rust, `crypto.test.ts`).

## Creating a chronolock

1. Generate the lock key.
2. Media (optional): `start_media_upload` returns an upload id. The file is split
   into segments that fit one canister message; each segment is encrypted
   separately with associated data `"chronolock/v2/media" || upload_id || index || count`,
   so chunks cannot be reordered, dropped or moved to another lock.
3. Metadata (title, description, attributes, file type) is encrypted with
   associated data `"chronolock/v2/meta"`. Titles are not stored in plaintext.
4. The lock key is IBE-encrypted to each identity (`user_keys`).
5. `create_chronolock` stores the ciphertexts with `scheme = 2`.

## Opening a chronolock

1. The browser creates a fresh transport key pair (`TransportSecretKey.random()`).
2. It calls `user_time_key` (recipient) or `public_time_key` (public) with the
   unlock time and the transport public key. The backend checks sign-in, the
   unlock time and a per-caller rate limit, then derives the vetKey.
3. The browser decrypts and verifies the vetKey against the offline-derived IBE
   public key, decrypts the lock key, then the metadata and each media chunk.

## What is public

Stored in plaintext on-chain: owner, creator, unlock time, creation time,
recipient principals (needed for "shared with me" lists), media size and chunk
count. Everything else is ciphertext.

## Limits and costs

- `vetkd_derive_key` costs cycles; each signed-in caller may request
  `key_derivations_per_hour` keys (default 60).
- Media is limited to `max_media_bytes` (default 10 MB) per lock, uploaded in
  chunks of at most `max_chunk_bytes`.
- The vetKD key name (`test_key_1` locally, `key_1` on mainnet) is fixed at
  install. Changing it, or the context, makes existing locks unreadable.
