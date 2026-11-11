import type { ChronoError, PaymentError } from '../bindings/backend';

type Variant<T extends string, V> = { __kind__: T } & Record<T, V>;
type Res<T> = Variant<'Ok', T> | Variant<'Err', ChronoError>;

export class ChronoApiError extends Error {
  constructor(public readonly error: ChronoError) {
    super(describeError(error));
    this.name = 'ChronoApiError';
  }
}

export function describeError(e: ChronoError): string {
  switch (e.__kind__) {
    case 'AnonymousCaller':
      return 'Please sign in first.';
    case 'Unauthorized':
      return 'You are not allowed to do that.';
    case 'NotFound':
      return 'Not found.';
    case 'TimeLocked':
      return 'This chronolock is still locked.';
    case 'AlreadyProcessing':
      return 'Another request is still in progress. Please wait.';
    case 'InvalidInput':
      return e.InvalidInput;
    case 'LimitExceeded':
      return e.LimitExceeded;
    case 'Payment':
      return describePayment(e.Payment);
    case 'Internal':
      return `Something went wrong: ${e.Internal}`;
  }
}

function describePayment(e: PaymentError): string {
  switch (e.__kind__) {
    case 'NotEnabled':
      return '$CHRONO payments are not enabled yet. Please pay with ICP.';
    case 'InsufficientFunds':
      return 'Your balance is too low for this payment.';
    case 'InsufficientAllowance':
      return 'The payment approval was not found or expired. Please try again.';
    case 'LedgerUnavailable':
      return 'The token ledger could not be reached. Nothing was charged; please try again.';
    case 'Rejected':
      return `The payment was rejected: ${e.Rejected}`;
  }
}

/** Returns the Ok value or throws a readable error. */
export function unwrap<T>(result: Res<T>): T {
  if (result.__kind__ === 'Ok') return result.Ok;
  throw new ChronoApiError(result.Err);
}

export function errorMessage(err: unknown): string {
  if (err instanceof Error) return err.message;
  return String(err);
}
