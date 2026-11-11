// @vitest-environment node
/**
 * End-to-end test against a local icp-cli deployment.
 *   icp network start -d && icp deploy --identity anonymous
 *   ICP_E2E=1 pnpm --dir frontend test e2e
 */
import { execFileSync } from 'node:child_process';
import { Ed25519KeyIdentity } from '@icp-sdk/core/identity';
import { describe, expect, it } from 'vitest';
import { decryptChronolock, getIbePublicKey, uploadMedia } from '../src/api/backend';
import { approve } from '../src/api/ledger';
import { createActor, PaymentMethod, type Backend } from '../src/bindings/backend';
import { createActor as createLedger } from '../src/bindings/icrc_ledger';
import { SCHEME_VERSION, encryptPayload, generateAesKey, wrapKey } from '../src/lib/crypto';
import { ChronoApiError, unwrap } from '../src/lib/errors';
import { icpAccountId, parseRecipient, sendTokens } from '../src/features/wallet/transfer';

const enabled = !!process.env.ICP_E2E;
const icp = (...args: string[]) =>
  execFileSync('icp', args, { encoding: 'utf8', cwd: '..' }).trim();

const backendCanisterId = () => icp('canister', 'status', 'backend', '--id-only');
const ibeKey = (_b: Backend) => getIbePublicKey(backendCanisterId(), 'test_key_1', 'local');

function actorFor(identity?: Ed25519KeyIdentity): Backend {
  const { api_url, root_key } = JSON.parse(icp('network', 'status', '--json'));
  return createActor(backendCanisterId(), {
    agentOptions: {
      host: api_url,
      rootKey: Uint8Array.from(Buffer.from(root_key, 'hex')),
      identity,
    },
  });
}

const sleep = (ms: number) => new Promise((r) => setTimeout(r, ms));

function agentOptions(identity?: Ed25519KeyIdentity) {
  const { api_url, root_key } = JSON.parse(icp('network', 'status', '--json'));
  return { host: api_url, rootKey: Uint8Array.from(Buffer.from(root_key, 'hex')), identity };
}

/** Sends local test ICP (anonymous is seeded by the local network). */
function fundIcp(principal: string, amount: string) {
  icp('token', 'transfer', amount, principal, '-e', 'local', '--identity', 'anonymous');
}

async function approveFor(backend: Backend, who: Ed25519KeyIdentity, method: PaymentMethod, withMedia: boolean) {
  const info = await backend.get_payment_info();
  const ledgerId = method === PaymentMethod.ICP ? info.icp_ledger : info.chrono_ledger!;
  const ledger = createLedger(ledgerId.toText(), { agentOptions: agentOptions(who) });
  const fee = await ledger.icrc1_fee();
  const p = info.prices;
  const price = method === PaymentMethod.ICP
    ? (withMedia ? p.icp_media : p.icp_text)
    : (withMedia ? p.chrono_media : p.chrono_text);
  await approve(ledger, info.treasury.owner, price, fee);
  return { ledger, price };
}

describe.skipIf(!enabled)('local deployment', () => {
  const alice = Ed25519KeyIdentity.generate();
  const bob = Ed25519KeyIdentity.generate();
  const aliceId = alice.getPrincipal().toText();
  const bobId = bob.getPrincipal().toText();

  it('rejects anonymous creators', async () => {
    const res = await actorFor().create_chronolock({
      scheme: SCHEME_VERSION,
      payment: PaymentMethod.ICP,
      unlock_time: 9_999_999_999n,
      user_keys: [{ user: 'public', key: 'x' }],
      encrypted_metadata: 'x',
    });
    expect(res.__kind__).toBe('Err');
  });

  it('fails without approval and charges nothing', async () => {
    fundIcp(aliceId, '5');
    const backend = actorFor(alice);
    const before = (await backend.get_payment_info()).collected.payments;
    const key = await generateAesKey();
    const unlock = BigInt(Math.floor(Date.now() / 1000) + 60);
    const res = await backend.create_chronolock({
      scheme: SCHEME_VERSION,
      payment: PaymentMethod.ICP,
      unlock_time: unlock,
      user_keys: await wrapKey(key, await ibeKey(backend), [], unlock),
      encrypted_metadata: await encryptPayload(key, { title: 'unpaid' }),
    });
    expect(res.__kind__).toBe('Err');
    const err = res.__kind__ === 'Err' ? res.Err : undefined;
    expect(err?.__kind__).toBe('Payment');
    expect(err?.__kind__ === 'Payment' && err.Payment.__kind__).toBe('InsufficientAllowance');
    expect((await backend.get_payment_info()).collected.payments).toBe(before);
  }, 60_000);

  it('time-locks a private text chronolock, then only the recipient decrypts it', async () => {
    const backend = actorFor(alice);
    const key = await generateAesKey();
    const unlock = BigInt(Math.floor(Date.now() / 1000) + 8);
    const { ledger, price } = await approveFor(backend, alice, PaymentMethod.ICP, false);
    const info = await backend.get_payment_info();
    const treasuryBefore = await ledger.icrc1_balance_of(info.treasury);
    const lock = unwrap(
      await backend.create_chronolock({
        scheme: SCHEME_VERSION,
        payment: PaymentMethod.ICP,
        unlock_time: unlock,
        user_keys: await wrapKey(key, await ibeKey(backend), [bobId], unlock),
        encrypted_metadata: await encryptPayload(key, { title: 'For Bob', description: 'secret' }),
      }),
    );
    expect(lock.owner.toText()).toBe(aliceId);
    expect(lock.payment?.amount).toBe(price);
    expect(await ledger.icrc1_balance_of(info.treasury)).toBe(treasuryBefore + price);

    const bobBackend = actorFor(bob);
    const pk = await ibeKey(backend);
    await expect(decryptChronolock(bobBackend, lock, bobId, pk)).rejects.toThrow(ChronoApiError);
    await expect(decryptChronolock(backend, lock, aliceId, pk)).rejects.toThrow(/not shared/);

    await sleep(10000);
    const out = await decryptChronolock(bobBackend, lock, bobId, pk);
    expect(out.title).toBe('For Bob');
    expect(out.description).toBe('secret');

    const received = await bobBackend.list_received_chronolocks(bob.getPrincipal(), true, {
      offset: 0n,
      limit: 10n,
    });
    expect(received.items.some((l) => l.id === lock.id)).toBe(true);
  }, 60_000);

  it('pays with $CHRONO and stores multi-chunk media for a public chronolock', async () => {
    const backend = actorFor(alice);
    const info = await backend.get_payment_info();
    const method = info.chrono_ledger ? PaymentMethod.CHRONO : PaymentMethod.ICP;
    if (info.chrono_ledger) {
      execFileSync('scripts/local-token.sh', ['mint', aliceId, '10000'], { cwd: '..' });
    }
    const key = await generateAesKey();
    const media = crypto.getRandomValues(new Uint8Array(65_536)).map((b) => b);
    const big = new Uint8Array(3_000_000);
    for (let i = 0; i < big.length; i += media.length) big.set(media.subarray(0, big.length - i), i);
    const mediaId = await uploadMedia(backend, key, big, 1_900_000);

    const unlock = BigInt(Math.floor(Date.now() / 1000) + 6);
    await approveFor(backend, alice, method, true);
    const lock = unwrap(
      await backend.create_chronolock({
        scheme: SCHEME_VERSION,
        payment: method,
        unlock_time: unlock,
        user_keys: await wrapKey(key, await ibeKey(backend), [], unlock),
        encrypted_metadata: await encryptPayload(key, { title: 'Public media', fileType: 'image/png' }),
        media_id: mediaId,
      }),
    );
    expect(lock.media?.chunk_count).toBe(2);
    expect(lock.payment?.method).toBe(method);

    await sleep(6000);
    const out = await decryptChronolock(actorFor(bob), lock, bobId, await ibeKey(backend));
    expect(out.mediaUrl).toBeTruthy();
    const bytes = new Uint8Array(await (await fetch(out.mediaUrl!)).arrayBuffer());
    expect(bytes).toEqual(big);
  }, 60_000);

  it('wallet: sends ICP by principal and by account id, and $CHRONO', async () => {
    fundIcp(aliceId, '3');
    const info = await actorFor().get_payment_info();
    const { identity: _i, ...overrides } = agentOptions();
    const icpLedger = createLedger(info.icp_ledger.toText(), { agentOptions: overrides });
    const bobBefore = await icpLedger.icrc1_balance_of({ owner: bob.getPrincipal() });

    await sendTokens({ ledger: info.icp_ledger, identity: alice, amount: 50_000_000n,
      to: parseRecipient(bobId, true), agentOverrides: overrides });
    await sendTokens({ ledger: info.icp_ledger, identity: alice, amount: 25_000_000n,
      to: parseRecipient(icpAccountId(bobId), true), agentOverrides: overrides });
    expect(await icpLedger.icrc1_balance_of({ owner: bob.getPrincipal() })).toBe(bobBefore + 75_000_000n);

    expect(() => parseRecipient(icpAccountId(bobId), false)).toThrow(/principal/);
    await expect(sendTokens({ ledger: info.icp_ledger, identity: alice, amount: 10n ** 12n,
      to: parseRecipient(bobId, true), agentOverrides: overrides })).rejects.toThrow(/Not enough/);

    if (info.chrono_ledger) {
      execFileSync('scripts/local-token.sh', ['mint', aliceId, '100'], { cwd: '..' });
      const chrono = createLedger(info.chrono_ledger.toText(), { agentOptions: overrides });
      const before = await chrono.icrc1_balance_of({ owner: bob.getPrincipal() });
      await sendTokens({ ledger: info.chrono_ledger, identity: alice, amount: 1_000_000_000n,
        to: parseRecipient(bobId, false), agentOverrides: overrides });
      expect(await chrono.icrc1_balance_of({ owner: bob.getPrincipal() })).toBe(before + 1_000_000_000n);
    }
  }, 60_000);

  it('derives the IBE key offline, matching the canister', async () => {
    const { deriveIbePublicKey } = await import('../src/lib/crypto');
    const offline = deriveIbePublicKey(backendCanisterId(), 'test_key_1', 'local');
    const fromCanister = unwrap(await actorFor(alice).ibe_public_key());
    expect(Buffer.from(offline.publicKeyBytes())).toEqual(Buffer.from(fromCanister));
  });
});
