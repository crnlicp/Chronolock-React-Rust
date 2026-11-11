import { Alert, Box, Button, Stack, ToggleButton, ToggleButtonGroup, Typography } from '@mui/material';
import type { Principal } from '@icp-sdk/core/principal';
import { Link as RouterLink } from 'react-router';
import { useBalance, useTokenMeta, type TokenMeta } from '../../api/ledger';
import { PaymentMethod, type PaymentInfo } from '../../bindings/backend';
import { formatUnits } from '../../lib/format';

export interface Quote {
  method: PaymentMethod;
  ledger: Principal;
  meta: TokenMeta;
  price: bigint;
  /** price + approve fee + payment fee */
  total: bigint;
  balance: bigint;
  enough: boolean;
}

export function priceFor(info: PaymentInfo, method: PaymentMethod, withMedia: boolean): bigint {
  const p = info.prices;
  if (method === PaymentMethod.ICP) return withMedia ? p.icp_media : p.icp_text;
  return withMedia ? p.chrono_media : p.chrono_text;
}

/** Price, fees and balance for paying with `method`. */
export function useQuote(info: PaymentInfo | undefined, method: PaymentMethod, withMedia: boolean): Quote | null {
  const ledger = info ? (method === PaymentMethod.ICP ? info.icp_ledger : info.chrono_ledger ?? undefined) : undefined;
  const { data: meta } = useTokenMeta(ledger);
  const { data: balance } = useBalance(ledger);
  if (!info || !ledger || !meta || balance === undefined) return null;
  const price = priceFor(info, method, withMedia);
  const total = price + 2n * meta.fee;
  return { method, ledger, meta, price, total, balance, enough: balance >= total };
}

interface Props {
  info: PaymentInfo;
  withMedia: boolean;
  method: PaymentMethod;
  onMethod: (m: PaymentMethod) => void;
  quote: Quote | null;
  disabled?: boolean;
}

function Row({ label, value, strong, color }: { label: string; value: string; strong?: boolean; color?: string }) {
  return (
    <Box sx={{ display: 'flex', justifyContent: 'space-between', gap: 2 }}>
      <Typography color="text.secondary">{label}</Typography>
      <Typography sx={{ fontWeight: strong ? 700 : 400, color }}>{value}</Typography>
    </Box>
  );
}

/** Lets the user pick ICP or $CHRONO and shows price, fees and balance. */
export const PaymentPicker = ({ info, withMedia, method, onMethod, quote, disabled }: Props) => {
  const chronoEnabled = !!info.chrono_ledger;
  const fmt = (v: bigint) => (quote ? `${formatUnits(v, quote.meta.decimals)} ${quote.meta.symbol}` : '…');
  return (
    <Box>
      <ToggleButtonGroup
        exclusive
        fullWidth
        value={method}
        disabled={disabled}
        onChange={(_, v) => v && onMethod(v)}
        sx={{ mb: 3 }}
      >
        <ToggleButton value={PaymentMethod.ICP}>Pay with ICP</ToggleButton>
        <ToggleButton value={PaymentMethod.CHRONO} disabled={!chronoEnabled}>
          {chronoEnabled ? 'Pay with $CHRONO' : '$CHRONO (coming soon)'}
        </ToggleButton>
      </ToggleButtonGroup>
      <Stack spacing={1}>
        <Row label={withMedia ? 'Media chronolock' : 'Text chronolock'} value={quote ? fmt(quote.price) : '…'} />
        <Row label="Network fees (approve + payment)" value={quote ? fmt(2n * quote.meta.fee) : '…'} />
        <Row label="Total" value={quote ? fmt(quote.total) : '…'} strong />
        <Row
          label="Your balance"
          value={quote ? fmt(quote.balance) : '…'}
          color={quote ? (quote.enough ? 'success.main' : 'error.main') : undefined}
        />
      </Stack>
      {quote && !quote.enough && (
        <Alert
          severity="warning"
          sx={{ mt: 2 }}
          action={
            <Button component={RouterLink} to="/wallet" color="inherit" size="small">
              Top up
            </Button>
          }
        >
          Not enough {quote.meta.symbol} for this chronolock.
        </Alert>
      )}
    </Box>
  );
};
