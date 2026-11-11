import {
  Alert,
  Box,
  Button,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  TextField,
  Typography,
} from '@mui/material';
import { useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { TokenMeta } from '../../api/ledger';
import { useAuth } from '../../auth/AuthProvider';
import { formatUnits, parseUnits } from '../../lib/format';
import { errorMessage } from '../../lib/errors';
import { parseRecipient, sendTokens, type Recipient } from './transfer';

interface Props {
  open: boolean;
  onClose: () => void;
  meta: TokenMeta;
  balance: bigint;
  isIcp: boolean;
}

export const SendDialog = ({ open, onClose, meta, balance, isIcp }: Props) => {
  const { identity } = useAuth();
  const queryClient = useQueryClient();
  const [to, setTo] = useState('');
  const [amount, setAmount] = useState('');
  const [review, setReview] = useState<{ recipient: Recipient; value: bigint } | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [block, setBlock] = useState<bigint | null>(null);
  const max = balance > meta.fee ? balance - meta.fee : 0n;
  const fmt = (v: bigint) => `${formatUnits(v, meta.decimals, meta.decimals)} ${meta.symbol}`;

  const reset = () => {
    setTo('');
    setAmount('');
    setReview(null);
    setError(null);
    setBlock(null);
  };
  const close = () => {
    if (busy) return;
    reset();
    onClose();
  };

  const onReview = () => {
    setError(null);
    try {
      const recipient = parseRecipient(to, isIcp);
      const value = parseUnits(amount, meta.decimals);
      if (value <= 0n) throw new Error('Enter an amount greater than zero.');
      if (value > max) throw new Error(`You can send at most ${fmt(max)}.`);
      setReview({ recipient, value });
    } catch (err) {
      setError(errorMessage(err));
    }
  };

  const onSend = async () => {
    if (!review || !identity) return;
    setBusy(true);
    setError(null);
    try {
      setBlock(
        await sendTokens({ ledger: meta.ledger, identity, to: review.recipient, amount: review.value }),
      );
      await queryClient.invalidateQueries({ queryKey: ['ledger'] });
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setBusy(false);
    }
  };

  return (
    <Dialog open={open} onClose={close} fullWidth maxWidth="sm" disableScrollLock>
      <DialogTitle>Send {meta.symbol}</DialogTitle>
      <DialogContent>
        {block !== null ? (
          <Alert severity="success" sx={{ mt: 1 }}>
            Sent {review && fmt(review.value)}. Block #{block.toString()}.
          </Alert>
        ) : review ? (
          <Box sx={{ display: 'grid', gridTemplateColumns: 'auto 1fr', gap: 1.5, mt: 1 }}>
            <Typography color="text.secondary">To</Typography>
            <Typography sx={{ wordBreak: 'break-all' }}>{to.trim()}</Typography>
            <Typography color="text.secondary">Amount</Typography>
            <Typography>{fmt(review.value)}</Typography>
            <Typography color="text.secondary">Network fee</Typography>
            <Typography>{fmt(meta.fee)}</Typography>
            <Typography color="text.secondary">Total</Typography>
            <Typography sx={{ fontWeight: 700 }}>{fmt(review.value + meta.fee)}</Typography>
          </Box>
        ) : (
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2, mt: 1 }}>
            <TextField
              label={isIcp ? 'Recipient principal or ICP account ID' : 'Recipient principal'}
              value={to}
              onChange={(e) => setTo(e.target.value)}
              autoFocus
              fullWidth
            />
            <TextField
              label={`Amount (${meta.symbol})`}
              value={amount}
              onChange={(e) => setAmount(e.target.value)}
              inputMode="decimal"
              fullWidth
              helperText={`Available ${fmt(max)} after the ${fmt(meta.fee)} fee`}
              slotProps={{
                input: {
                  endAdornment: (
                    <Button size="small" onClick={() => setAmount(formatUnits(max, meta.decimals, meta.decimals).replace(/,/g, ''))}>
                      Max
                    </Button>
                  ),
                },
              }}
            />
          </Box>
        )}
        {error && (
          <Alert severity="error" sx={{ mt: 2 }}>
            {error}
          </Alert>
        )}
      </DialogContent>
      <DialogActions>
        {block !== null ? (
          <Button onClick={close}>Done</Button>
        ) : review ? (
          <>
            <Button onClick={() => setReview(null)} disabled={busy}>
              Back
            </Button>
            <Button variant="contained" onClick={onSend} disabled={busy}>
              {busy ? 'Sending…' : 'Confirm & send'}
            </Button>
          </>
        ) : (
          <>
            <Button onClick={close}>Cancel</Button>
            <Button variant="contained" onClick={onReview} disabled={!to || !amount}>
              Review
            </Button>
          </>
        )}
      </DialogActions>
    </Dialog>
  );
};
