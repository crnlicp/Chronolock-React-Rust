// @vitest-environment node
import { Principal } from '@icp-sdk/core/principal';
import { describe, expect, it } from 'vitest';
import type { Chronolock } from '../src/bindings/backend';
import { viewOf } from '../src/features/collection/status';
import { emptyDraft, recipientsOf } from '../src/features/create/types';
import { formatUnits, parseUnits } from '../src/lib/format';

const me = 'aaaaa-aa';
const alice = Principal.fromUint8Array(new Uint8Array([1, 2, 3])).toText();

describe('format', () => {
  it('formats and parses token amounts', () => {
    expect(formatUnits(120_000_000n, 8)).toBe('1.2');
    expect(formatUnits(500_000_000_000n, 8)).toBe('5,000');
    expect(formatUnits(10_000n, 8, 8)).toBe('0.0001');
    expect(parseUnits('0.2', 8)).toBe(20_000_000n);
    expect(parseUnits('1,000', 8)).toBe(100_000_000_000n);
    expect(() => parseUnits('1.123456789', 8)).toThrow();
    expect(() => parseUnits('abc', 8)).toThrow();
  });
});

describe('recipientsOf', () => {
  it('maps the audience choice to recipients', () => {
    expect(recipientsOf({ ...emptyDraft, audience: 'me' }, me)).toEqual([me]);
    expect(recipientsOf({ ...emptyDraft, audience: 'public' }, me)).toEqual([]);
    expect(recipientsOf({ ...emptyDraft, audience: 'people', people: [alice], includeMe: true }, me)).toEqual([alice, me]);
    expect(recipientsOf({ ...emptyDraft, audience: 'people', people: [alice, alice], includeMe: false }, me)).toEqual([alice]);
  });
});

describe('viewOf', () => {
  const lock = (users: string[], owner = me) =>
    ({ owner: Principal.fromText(owner), user_keys: users.map((user) => ({ user, key: '' })) }) as unknown as Chronolock;

  it('knows who can open a lock', () => {
    expect(viewOf(lock(['public']), alice)).toMatchObject({ isPublic: true, canOpen: true, isOwner: false });
    expect(viewOf(lock([alice]), alice)).toMatchObject({ canOpen: true, recipientCount: 1 });
    expect(viewOf(lock([alice]), me)).toMatchObject({ canOpen: false, isOwner: true });
    expect(viewOf(lock(['public']), undefined).canOpen).toBe(false);
  });
});
