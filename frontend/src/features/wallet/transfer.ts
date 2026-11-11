import { AccountIdentifier, IcpLedgerCanister } from '@icp-sdk/canisters/ledger/icp';
import { decodeIcrcAccount, type IcrcAccount } from '@icp-sdk/canisters/ledger/icrc';
import { HttpAgent, type HttpAgentOptions, type Identity } from '@icp-sdk/core/agent';
import { Principal } from '@icp-sdk/core/principal';
import { ledgerActor } from '../../api/ledger';
import { rootKey } from '../../lib/env';

export type Recipient =
  | { kind: 'icrc'; account: IcrcAccount }
  /** Legacy 64-hex ICP account identifier (exchanges use these). */
  | { kind: 'accountId'; accountId: AccountIdentifier };

/**
 * Parses a principal, an ICRC-1 account text, or (ICP only) a 64-hex account
 * identifier. Throws a readable error otherwise.
 */
export function parseRecipient(text: string, allowAccountId: boolean): Recipient {
  const v = text.trim();
  if (/^[0-9a-f]{64}$/i.test(v)) {
    if (!allowAccountId) throw new Error('This token only accepts a principal or ICRC account.');
    return { kind: 'accountId', accountId: AccountIdentifier.fromHex(v.toLowerCase()) };
  }
  try {
    const account = decodeIcrcAccount(v);
    if (account.owner.isAnonymous()) throw new Error();
    return { kind: 'icrc', account };
  } catch {
    throw new Error('Enter a valid principal or account.');
  }
}

/** The legacy ICP account id of a principal's default account (for exchanges). */
export function icpAccountId(principal: string): string {
  return AccountIdentifier.fromPrincipal({ principal: Principal.fromText(principal) }).toHex();
}

/** Sends `amount` (smallest units) from the signed-in user's default account. */
export async function sendTokens(opts: {
  ledger: Principal;
  identity: Identity;
  to: Recipient;
  amount: bigint;
  /** Only for tests/scripts outside the browser (host, rootKey). */
  agentOverrides?: HttpAgentOptions;
}): Promise<bigint> {
  const createdAt = BigInt(Date.now()) * 1_000_000n;
  if (opts.to.kind === 'accountId') {
    const agent = await HttpAgent.create({ identity: opts.identity, rootKey, ...opts.agentOverrides });
    const icp = IcpLedgerCanister.create({ agent, canisterId: opts.ledger });
    return icp.transfer({ to: opts.to.accountId, amount: opts.amount, createdAt });
  }
  const res = await ledgerActor(opts.ledger, opts.identity, opts.agentOverrides).icrc1_transfer({
    to: { owner: opts.to.account.owner, subaccount: opts.to.account.subaccount },
    amount: opts.amount,
    created_at_time: createdAt,
  });
  if (res.__kind__ === 'Ok') return res.Ok;
  const e = res.Err;
  switch (e.__kind__) {
    case 'InsufficientFunds':
      throw new Error('Not enough balance (remember the network fee).');
    case 'BadFee':
      throw new Error('The network fee changed. Please try again.');
    case 'TemporarilyUnavailable':
      throw new Error('The ledger is temporarily unavailable. Please try again.');
    case 'GenericError':
      throw new Error(e.GenericError.message);
    default:
      throw new Error(`Transfer failed (${e.__kind__}).`);
  }
}
