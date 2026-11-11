import { Alert, Box, Button, Card, CardContent, Grid, LinearProgress, Stack, Typography } from '@mui/material';
import { useQueryClient } from '@tanstack/react-query';
import dayjs from 'dayjs';
import { useState } from 'react';
import { ibePublicKeyFor, keys, useBackend } from '../../api/backend';
import { approve, ledgerActor, usePaymentInfo } from '../../api/ledger';
import { useAuth } from '../../auth/AuthProvider';
import { PaymentMethod } from '../../bindings/backend';
import { SCHEME_VERSION, encryptPayload, wrapKey } from '../../lib/crypto';
import { errorMessage, unwrap } from '../../lib/errors';
import { PaymentPicker, useQuote } from './PaymentPicker';
import { recipientsOf, type Draft } from './types';

interface Props {
  draft: Draft;
  aesKey: CryptoKey;
  onBack: () => void;
  onCreated: (id: bigint) => void;
}

function Item({ label, children }: { label: string; children: React.ReactNode }) {
  return (
    <Box sx={{ mb: 2 }}>
      <Typography variant="caption" color="text.secondary">
        {label}
      </Typography>
      <Box>{children}</Box>
    </Box>
  );
}

export const ReviewStep = ({ draft, aesKey, onBack, onCreated }: Props) => {
  const { principal, identity } = useAuth();
  const backend = useBackend();
  const queryClient = useQueryClient();
  const { data: info } = usePaymentInfo();
  const [method, setMethod] = useState<PaymentMethod>(PaymentMethod.ICP);
  const withMedia = !!draft.mediaId;
  const quote = useQuote(info, method, withMedia);
  const [step, setStep] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const recipients = principal ? recipientsOf(draft, principal) : [];

  const create = async () => {
    if (!principal || !identity || !info || !quote?.enough || !draft.unlockAt) return;
    setError(null);
    try {
      const unlock = BigInt(draft.unlockAt);
      if (draft.unlockAt <= Date.now() / 1000 + 30) throw new Error('The unlock time has passed. Go back and pick a new one.');
      setStep('Sealing your content…');
      const encrypted_metadata = await encryptPayload(aesKey, {
        title: draft.title.trim(),
        description: draft.message,
        fileName: draft.file?.name,
        fileType: draft.file?.type || undefined,
        mediaSize: draft.file?.size,
      });
      const user_keys = await wrapKey(aesKey, await ibePublicKeyFor(backend), recipients, unlock);
      setStep(`Approving ${quote.meta.symbol} payment…`);
      await approve(ledgerActor(quote.ledger, identity), info.treasury.owner, quote.price, quote.meta.fee);
      setStep('Paying and creating your chronolock…');
      const lock = unwrap(
        await backend.create_chronolock({
          scheme: SCHEME_VERSION,
          payment: method,
          unlock_time: unlock,
          user_keys,
          encrypted_metadata,
          media_id: draft.mediaId ? BigInt(draft.mediaId) : undefined,
        }),
      );
      await queryClient.invalidateQueries({ queryKey: keys.all });
      await queryClient.invalidateQueries({ queryKey: ['ledger'] });
      onCreated(lock.id);
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setStep(null);
    }
  };

  const busy = step !== null;
  const unlock = draft.unlockAt ? dayjs.unix(draft.unlockAt) : null;

  return (
    <Grid container spacing={4}>
      <Grid size={{ xs: 12, md: 6 }}>
        <Typography variant="h6" gutterBottom>
          Summary
        </Typography>
        <Item label="Title">
          <Typography>{draft.title}</Typography>
        </Item>
        <Item label="Opens">
          <Typography>
            {unlock?.format('dddd, D MMMM YYYY [at] HH:mm')} ({unlock?.fromNow()})
          </Typography>
        </Item>
        <Item label="Who can open it">
          <Typography>
            {recipients.length === 0
              ? 'Everyone (public)'
              : recipients.length === 1 && recipients[0] === principal
                ? 'Only you'
                : `${recipients.length} ${recipients.length === 1 ? 'person' : 'people'}${recipients.includes(principal ?? '') ? ', including you' : ''}`}
          </Typography>
        </Item>
        <Item label="Attachment">
          <Typography>{draft.file ? `${draft.file.name}` : 'None'}</Typography>
        </Item>
        <Alert severity="info" sx={{ mt: 2 }}>
          Once sealed, the content and unlock time cannot be changed, and nobody can open it early.
        </Alert>
      </Grid>
      <Grid size={{ xs: 12, md: 6 }}>
        <Card>
          <CardContent sx={{ p: 3 }}>
            <Typography variant="h6" gutterBottom>
              Payment
            </Typography>
            {info ? (
              <PaymentPicker info={info} withMedia={withMedia} method={method} onMethod={setMethod} quote={quote} disabled={busy} />
            ) : (
              <LinearProgress />
            )}
            {step && (
              <Box sx={{ mt: 3 }}>
                <Typography variant="body2" gutterBottom>
                  {step}
                </Typography>
                <LinearProgress />
              </Box>
            )}
            {error && (
              <Alert severity="error" sx={{ mt: 3 }}>
                {error}
              </Alert>
            )}
            <Stack direction="row" sx={{ justifyContent: 'space-between', mt: 3 }}>
              <Button onClick={onBack} disabled={busy}>
                Back
              </Button>
              <Button variant="contained" size="large" onClick={create} disabled={busy || !quote?.enough}>
                Seal & pay
              </Button>
            </Stack>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );
};
