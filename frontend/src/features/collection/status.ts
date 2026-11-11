import type { Chronolock } from '../../bindings/backend';
import { PUBLIC_RECIPIENT } from '../../lib/crypto';

export interface LockView {
  isPublic: boolean;
  isOwner: boolean;
  /** The viewer is a recipient (or it is public). */
  canOpen: boolean;
  recipientCount: number;
}

export function viewOf(lock: Chronolock, principal?: string): LockView {
  const isPublic = lock.user_keys.some((k) => k.user === PUBLIC_RECIPIENT);
  const mine = !!principal && lock.user_keys.some((k) => k.user === principal);
  return {
    isPublic,
    isOwner: !!principal && lock.owner.toText() === principal,
    canOpen: !!principal && (isPublic || mine),
    recipientCount: lock.user_keys.filter((k) => k.user !== PUBLIC_RECIPIENT).length,
  };
}
