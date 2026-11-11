/** Formats a smallest-unit amount, e.g. 120000000n with 8 decimals -> "1.2". */
export function formatUnits(amount: bigint, decimals: number, maxFraction = 4): string {
  const neg = amount < 0n;
  const abs = neg ? -amount : amount;
  const base = 10n ** BigInt(decimals);
  const whole = (abs / base).toLocaleString('en-US');
  let frac = (abs % base).toString().padStart(decimals, '0').slice(0, maxFraction);
  frac = frac.replace(/0+$/, '');
  return `${neg ? '-' : ''}${whole}${frac ? `.${frac}` : ''}`;
}

/** Parses a decimal string into smallest units; throws on invalid input. */
export function parseUnits(value: string, decimals: number): bigint {
  const v = value.trim().replace(/,/g, '');
  if (!/^\d+(\.\d+)?$/.test(v)) throw new Error('Enter a valid amount');
  const [whole, frac = ''] = v.split('.');
  if (frac.length > decimals) throw new Error(`At most ${decimals} decimals`);
  return BigInt(whole) * 10n ** BigInt(decimals) + BigInt(frac.padEnd(decimals, '0') || '0');
}
