export type Audience = 'me' | 'people' | 'public';

export interface Draft {
  unlockAt: number | null; // unix seconds
  audience: Audience;
  people: string[];
  includeMe: boolean;
  title: string;
  message: string;
  file: File | null;
  /** Upload id once the file is encrypted and uploaded with `key`. */
  mediaId: string | null;
}

export const emptyDraft: Draft = {
  unlockAt: null,
  audience: 'me',
  people: [],
  includeMe: true,
  title: '',
  message: '',
  file: null,
  mediaId: null,
};

/** Principals that can open the lock; empty means public. */
export function recipientsOf(d: Draft, me: string): string[] {
  if (d.audience === 'public') return [];
  if (d.audience === 'me') return [me];
  const set = new Set(d.people);
  if (d.includeMe) set.add(me);
  return [...set];
}
