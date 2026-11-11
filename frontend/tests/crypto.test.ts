// @vitest-environment node
import { Principal } from '@icp-sdk/core/principal';
import { describe, expect, it } from 'vitest';
import {
  aesDecrypt,
  aesEncrypt,
  decryptPayload,
  encryptPayload,
  generateAesKey,
  ibeIdentity,
  mediaChunkAad,
} from '../src/lib/crypto';

describe('scheme v2 crypto', () => {
  it('builds the same IBE identities as the backend', () => {
    // Mirrors backend/src/vetkd.rs `identity_layout` test.
    expect([...ibeIdentity(1n)]).toEqual([0x43, 0x4c, 2, 0, 0, 0, 0, 0, 0, 0, 0, 1]);
    const p = Principal.fromUint8Array(new Uint8Array([7, 8, 9])).toText();
    expect([...ibeIdentity(256n, p)]).toEqual([
      0x43, 0x4c, 2, 1, 0, 0, 0, 0, 0, 0, 1, 0, 3, 7, 8, 9,
    ]);
    expect(ibeIdentity(5n, 'public')).toEqual(ibeIdentity(5n));
  });

  it('round-trips the metadata payload', async () => {
    const key = await generateAesKey();
    const enc = await encryptPayload(key, { title: 'Hello', description: 'x' });
    expect(await decryptPayload(key, enc)).toEqual({ title: 'Hello', description: 'x' });
  });

  it('rejects a media chunk moved to another position or upload', async () => {
    const key = await generateAesKey();
    const data = new Uint8Array([1, 2, 3]);
    const ct = await aesEncrypt(key, data, mediaChunkAad(7n, 0, 2));
    expect(await aesDecrypt(key, ct, mediaChunkAad(7n, 0, 2))).toEqual(data);
    await expect(aesDecrypt(key, ct, mediaChunkAad(7n, 1, 2))).rejects.toThrow();
    await expect(aesDecrypt(key, ct, mediaChunkAad(8n, 0, 2))).rejects.toThrow();
    await expect(aesDecrypt(key, ct, mediaChunkAad(7n, 0, 3))).rejects.toThrow();
  });

  it('does not accept media ciphertext as metadata', async () => {
    const key = await generateAesKey();
    const ct = await aesEncrypt(key, new TextEncoder().encode('{}'), mediaChunkAad(1n, 0, 1));
    const b64 = btoa(String.fromCharCode(...ct));
    await expect(decryptPayload(key, b64)).rejects.toThrow();
  });
});
