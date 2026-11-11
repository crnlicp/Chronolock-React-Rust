import type { HttpAgentOptions, Identity } from '@icp-sdk/core/agent';
import { Principal } from '@icp-sdk/core/principal';
import { useQuery } from '@tanstack/react-query';
import { useAuth } from '../auth/AuthProvider';
import { createActor, type Icrc_ledger as Ledger } from '../bindings/icrc_ledger';
import { rootKey } from '../lib/env';
import { useBackend } from './backend';

export type LedgerActor = Ledger;

export function ledgerActor(
  ledgerId: Principal | string,
  identity?: Identity,
  overrides?: HttpAgentOptions,
): LedgerActor {
  return createActor(ledgerId.toString(), { agentOptions: { identity, rootKey, ...overrides } });
}

export interface TokenMeta {
  ledger: Principal;
  symbol: string;
  decimals: number;
  fee: bigint;
}

export const ledgerKeys = {
  meta: (ledger: string) => ['ledger', ledger, 'meta'] as const,
  balance: (ledger: string, owner: string) => ['ledger', ledger, 'balance', owner] as const,
};

/** Symbol, decimals and fee, read from the ledger (fees can change). */
export function useTokenMeta(ledger: Principal | undefined) {
  return useQuery({
    queryKey: ledgerKeys.meta(ledger?.toText() ?? ''),
    enabled: !!ledger,
    staleTime: 5 * 60_000,
    queryFn: async (): Promise<TokenMeta> => {
      const l = ledgerActor(ledger!);
      const [symbol, decimals, fee] = await Promise.all([
        l.icrc1_symbol(),
        l.icrc1_decimals(),
        l.icrc1_fee(),
      ]);
      return { ledger: ledger!, symbol, decimals, fee };
    },
  });
}

/** Balance of the signed-in user's default account. Queries go straight to the ledger. */
export function useBalance(ledger: Principal | undefined) {
  const { principal } = useAuth();
  return useQuery({
    queryKey: ledgerKeys.balance(ledger?.toText() ?? '', principal ?? ''),
    enabled: !!ledger && !!principal,
    refetchInterval: 30_000,
    queryFn: () =>
      ledgerActor(ledger!).icrc1_balance_of({ owner: Principal.fromText(principal!) }),
  });
}

export function usePaymentInfo() {
  const backend = useBackend();
  return useQuery({
    queryKey: ['backend', 'payment-info'],
    queryFn: () => backend.get_payment_info(),
    staleTime: 60_000,
  });
}

const APPROVAL_TTL_NS = 10n * 60n * 1_000_000_000n;

/**
 * ICRC-2 approve of `amount` (+ the transfer fee the spender's pull costs)
 * for `spender`, valid for 10 minutes.
 */
export async function approve(
  ledger: LedgerActor,
  spender: Principal,
  amount: bigint,
  fee: bigint,
): Promise<bigint> {
  const now = BigInt(Date.now()) * 1_000_000n;
  const res = await ledger.icrc2_approve({
    spender: { owner: spender },
    amount: amount + fee,
    expires_at: now + APPROVAL_TTL_NS,
    created_at_time: now,
  });
  if (res.__kind__ === 'Ok') return res.Ok;
  const e = res.Err;
  switch (e.__kind__) {
    case 'InsufficientFunds':
      throw new Error('Not enough balance to cover the approval fee.');
    case 'BadFee':
      throw new Error('The token fee changed. Please try again.');
    case 'TemporarilyUnavailable':
      throw new Error('The ledger is temporarily unavailable. Please try again.');
    case 'GenericError':
      throw new Error(e.GenericError.message);
    default:
      throw new Error(`Approval failed (${e.__kind__}).`);
  }
}
