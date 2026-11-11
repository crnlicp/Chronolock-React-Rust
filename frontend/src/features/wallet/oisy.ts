/**
 * OISY as a funding source (ICRC signer standards via @icp-sdk/signer).
 * Internet Identity stays the app account; OISY sends tokens into it with one
 * user-approved transfer. OISY lives on mainnet, so this only works there.
 */
import { IcrcLedgerCanister, toCandidAccount, type IcrcAccount } from '@icp-sdk/canisters/ledger/icrc';
import { HttpAgent } from '@icp-sdk/core/agent';
import type { Principal } from '@icp-sdk/core/principal';
import { Signer, SignerError } from '@icp-sdk/signer';
import { SignerAgent } from '@icp-sdk/signer/agent';
import { PostMessageTransport, PostMessageTransportError } from '@icp-sdk/signer/web';

const OISY_SIGN_URL = 'https://oisy.com/sign';
const MAINNET_API = 'https://icp-api.io';

let signer: Signer | undefined;
const getSigner = () =>
  (signer ??= new Signer({ transport: new PostMessageTransport({ url: OISY_SIGN_URL }) }));

/** Opens OISY and returns the accounts the user chose to share (may be empty). Call from a click. */
export async function connectOisy(): Promise<IcrcAccount[]> {
  return getSigner().getAccounts();
}

export function forgetOisy() {
  signer = undefined;
}

/** Public balance read: plain anonymous agent, no wallet prompt. */
export async function oisyBalance(ledger: Principal, account: IcrcAccount): Promise<bigint> {
  const agent = await HttpAgent.create({ host: MAINNET_API });
  return IcrcLedgerCanister.create({ agent, canisterId: ledger }).balance(account);
}

/** Transfers from the OISY account to `to`. Prompts the user in OISY. Call from a click. */
export async function oisyTransfer(opts: {
  from: IcrcAccount;
  ledger: Principal;
  to: Principal;
  amount: bigint;
}): Promise<bigint> {
  const agent = await HttpAgent.create({ host: MAINNET_API });
  const signerAgent = await SignerAgent.create({ signer: getSigner(), account: opts.from.owner, agent });
  const ledger = IcrcLedgerCanister.create({ agent: signerAgent, canisterId: opts.ledger });
  return ledger.transfer({
    to: toCandidAccount({ owner: opts.to }),
    from_subaccount: opts.from.subaccount,
    amount: opts.amount,
  });
}

/** Readable message for OISY failures; `null` when the user simply cancelled. */
export function oisyErrorMessage(err: unknown): string | null {
  if (!(err instanceof SignerError)) return err instanceof Error ? err.message : String(err);
  if (err.code === 3001) return null;
  if (err.code === 3000) return 'OISY permission was denied. Allow it in OISY and try again.';
  if (err.code === 2000) return 'OISY does not support this action.';
  if (Math.floor(err.code / 1000) === 4) {
    return err.cause instanceof PostMessageTransportError
      ? 'The OISY window was blocked or closed. Allow pop-ups and try again.'
      : 'Lost the connection to OISY. Please reconnect.';
  }
  if (Math.floor(err.code / 1000) === 3) return 'The action was not completed in OISY.';
  return err.message;
}
