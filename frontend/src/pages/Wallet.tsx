import RefreshIcon from '@mui/icons-material/Refresh';
import {
  Box,
  Button,
  Card,
  CardContent,
  Grid,
  IconButton,
  Link,
  Skeleton,
  ToggleButton,
  ToggleButtonGroup,
  Typography,
} from '@mui/material';
import type { Principal } from '@icp-sdk/core/principal';
import { QRCodeSVG } from 'qrcode.react';
import { useCallback, useEffect, useState } from 'react';
import { useBackendConfig } from '../api/backend';
import { useBalance, usePaymentInfo, useTokenMeta, type TokenMeta } from '../api/ledger';
import { useAuth } from '../auth/AuthProvider';
import { CopyField } from '../components/ui/CopyField';
import { PageHeader } from '../components/ui/PageHeader';
import { SignInPrompt } from '../components/ui/SignInPrompt';
import { OisyCard } from '../features/wallet/OisyCard';
import { SendDialog } from '../features/wallet/SendDialog';
import { icpAccountId } from '../features/wallet/transfer';
import { formatUnits } from '../lib/format';

const SOON_TOKEN_URL = 'https://soonlaunchpad.fun/t/';

function BalanceCard({
  ledger,
  isIcp,
  buyUrl,
  onMeta,
}: {
  ledger: Principal;
  isIcp: boolean;
  buyUrl?: string;
  onMeta: (m: TokenMeta) => void;
}) {
  const { data: meta } = useTokenMeta(ledger);
  const { data: balance, refetch, isFetching } = useBalance(ledger);
  const [sending, setSending] = useState(false);

  useEffect(() => {
    if (meta) onMeta(meta);
  }, [meta, onMeta]);

  return (
    <Card sx={{ height: '100%' }}>
      <CardContent sx={{ p: 3 }}>
        <Box sx={{ display: 'flex', justifyContent: 'space-between', alignItems: 'center' }}>
          <Typography variant="overline" color="text.secondary">
            {meta?.symbol ?? '…'} balance
          </Typography>
          <IconButton size="small" onClick={() => refetch()} disabled={isFetching} aria-label="Refresh balance">
            <RefreshIcon fontSize="small" />
          </IconButton>
        </Box>
        <Typography variant="h3" sx={{ my: 1 }}>
          {meta && balance !== undefined ? formatUnits(balance, meta.decimals) : <Skeleton width={160} />}
        </Typography>
        <Typography variant="caption" color="text.secondary">
          Network fee {meta ? `${formatUnits(meta.fee, meta.decimals, meta.decimals)} ${meta.symbol}` : '…'}
        </Typography>
        <Box sx={{ display: 'flex', gap: 1, mt: 3 }}>
          <Button
            variant="contained"
            disabled={!meta || balance === undefined || balance <= (meta?.fee ?? 0n)}
            onClick={() => setSending(true)}
          >
            Send
          </Button>
          {buyUrl && (
            <Button variant="outlined" href={buyUrl} target="_blank" rel="noopener noreferrer">
              Buy on Soon
            </Button>
          )}
        </Box>
      </CardContent>
      {meta && balance !== undefined && (
        <SendDialog open={sending} onClose={() => setSending(false)} meta={meta} balance={balance} isIcp={isIcp} />
      )}
    </Card>
  );
}

export const WalletPage = () => {
  const { principal, isAuthenticated } = useAuth();
  const { data: payment } = usePaymentInfo();
  const { data: config } = useBackendConfig();
  const [qr, setQr] = useState<'principal' | 'account'>('principal');
  const [metas, setMetas] = useState<Record<string, TokenMeta>>({});
  const addMeta = useCallback(
    (m: TokenMeta) =>
      setMetas((prev) => (prev[m.ledger.toText()] ? prev : { ...prev, [m.ledger.toText()]: m })),
    [],
  );

  if (!isAuthenticated || !principal) return <SignInPrompt message="Sign in to see your wallet." />;

  const accountId = icpAccountId(principal);
  const market = config?.token?.market ?? undefined;

  return (
    <>
      <PageHeader
        title="Wallet"
        subtitle="Your Chronolock wallet belongs to your Internet Identity. Use it to pay for chronolocks and to send or receive ICP and $CHRONO."
      />
      <Grid container spacing={3}>
        <Grid size={{ xs: 12, md: 6 }}>
          {payment ? <BalanceCard ledger={payment.icp_ledger} isIcp onMeta={addMeta} /> : <Skeleton variant="rounded" height={200} />}
        </Grid>
        <Grid size={{ xs: 12, md: 6 }}>
          {payment?.chrono_ledger ? (
            <BalanceCard
              ledger={payment.chrono_ledger}
              isIcp={false}
              buyUrl={market ? `${SOON_TOKEN_URL}${market.toText()}` : undefined}
              onMeta={addMeta}
            />
          ) : (
            <Card sx={{ height: '100%' }}>
              <CardContent sx={{ p: 3 }}>
                <Typography variant="overline" color="text.secondary">
                  $CHRONO
                </Typography>
                <Typography sx={{ mt: 1 }}>
                  $CHRONO is launching soon on{' '}
                  <Link href="https://soonlaunchpad.fun" target="_blank" rel="noopener noreferrer">
                    Soon
                  </Link>
                  .
                </Typography>
              </CardContent>
            </Card>
          )}
        </Grid>
        <Grid size={{ xs: 12, md: 6 }}>
          <Card sx={{ height: '100%' }}>
            <CardContent sx={{ p: 3 }}>
              <Typography variant="h6" gutterBottom>
                Receive
              </Typography>
              <CopyField label="Principal (ICP and $CHRONO)" value={principal} />
              <CopyField label="ICP account ID (for exchanges)" value={accountId} />
              <ToggleButtonGroup size="small" exclusive value={qr} onChange={(_, v) => v && setQr(v)} sx={{ mb: 2 }}>
                <ToggleButton value="principal">Principal</ToggleButton>
                <ToggleButton value="account">Account ID</ToggleButton>
              </ToggleButtonGroup>
              <Box sx={{ background: '#fff', p: 2, width: 'fit-content', borderRadius: 2 }}>
                <QRCodeSVG value={qr === 'principal' ? principal : accountId} size={168} />
              </Box>
            </CardContent>
          </Card>
        </Grid>
        <Grid size={{ xs: 12, md: 6 }}>
          <OisyCard tokens={Object.values(metas)} />
        </Grid>
      </Grid>
    </>
  );
};
