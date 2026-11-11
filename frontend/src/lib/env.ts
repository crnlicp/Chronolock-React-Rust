import { safeGetCanisterEnv } from '@icp-sdk/core/agent/canister-env';

type CanisterName = 'backend' | 'frontend';

/**
 * Canister ids and the network root key come from the `ic_env` cookie, set by
 * the frontend canister in production and by the Vite dev server locally.
 */
const env = safeGetCanisterEnv<Record<`PUBLIC_CANISTER_ID:${CanisterName}`, string>>();

export const rootKey: Uint8Array | undefined = env?.IC_ROOT_KEY;

export function canisterId(name: CanisterName): string | undefined {
  return env?.[`PUBLIC_CANISTER_ID:${name}`];
}

/** True when the page is served from a local icp-cli network. */
export const isLocal =
  typeof window !== 'undefined' &&
  /(^|\.)localhost$|^127\.0\.0\.1$/.test(window.location.hostname);

/**
 * Origin Internet Identity derives principals from on mainnet: the frontend
 * canister's own address. chronolock.app (listed in the canister's
 * /.well-known/ii-alternative-origins) uses it too, so users have the same
 * account on both, and a domain change can never orphan accounts.
 * Undefined locally, where the page origin is used.
 */
export function iiDerivationOrigin(): string | undefined {
  if (isLocal) return undefined;
  const id = canisterId('frontend');
  return id ? `https://${id}.icp0.io` : undefined;
}
