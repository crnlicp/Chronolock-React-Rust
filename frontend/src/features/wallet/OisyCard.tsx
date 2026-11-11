import { encodeIcrcAccount, type IcrcAccount } from '@icp-sdk/canisters/ledger/icrc';
import { Principal } from '@icp-sdk/core/principal';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  MenuItem,
  TextField,
  ToggleButton,
  ToggleButtonGroup,
  Typography,
} from '@mui/material';
import { useQuery, useQueryClient } from '@tanstack/react-query';
import { useState } from 'react';
import type { TokenMeta } from '../../api/ledger';
import { useAuth } from '../../auth/AuthProvider';
import { isLocal } from '../../lib/env';
import { formatUnits, parseUnits } from '../../lib/format';
import { connectOisy, forgetOisy, oisyBalance, oisyErrorMessage, oisyTransfer } from './oisy';

interface Props {
  tokens: TokenMeta[];
}

/** Top up the Chronolock (Internet Identity) wallet from OISY. */
export const OisyCard = ({ tokens }: Props) => {
  const { principal } = useAuth();
  const queryClient = useQueryClient();
  const [accounts, setAccounts] = useState<IcrcAccount[] | null>(null);
  const [selected, setSelected] = useState<string>('');
  const [ledgerId, setLedgerId] = useState<string>(tokens[0]?.ledger.toText() ?? '');
  const [amount, setAmount] = useState('');
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState<{ ok: boolean; text: string } | null>(null);

  const account = accounts?.find((a) => encodeIcrcAccount(a) === selected);
  const token = tokens.find((t) => t.ledger.toText() === ledgerId) ?? tokens[0];
  const { data: oisyBal } = useQuery({
    queryKey: ['oisy', selected, ledgerId],
    enabled: !!account && !!token,
    queryFn: () => oisyBalance(token!.ledger, account!),
  });

  const connect = async () => {
    setMessage(null);
    try {
      const list = await connectOisy();
      setAccounts(list);
      if (list.length === 0) setMessage({ ok: false, text: 'No account was shared from OISY.' });
      else setSelected(encodeIcrcAccount(list[0]));
    } catch (err) {
      const text = oisyErrorMessage(err);
      if (text) setMessage({ ok: false, text });
    }
  };

  const topUp = async () => {
    if (!account || !token || !principal) return;
    setMessage(null);
    let value: bigint;
    try {
      value = parseUnits(amount, token.decimals);
      if (value <= 0n) throw new Error('Enter an amount greater than zero.');
    } catch (err) {
      setMessage({ ok: false, text: (err as Error).message });
      return;
    }
    setBusy(true);
    try {
      const block = await oisyTransfer({
        from: account,
        ledger: token.ledger,
        to: Principal.fromText(principal),
        amount: value,
      });
      setMessage({ ok: true, text: `Topped up ${amount} ${token.symbol} (block #${block}).` });
      setAmount('');
      await queryClient.invalidateQueries({ queryKey: ['ledger'] });
      await queryClient.invalidateQueries({ queryKey: ['oisy'] });
    } catch (err) {
      const text = oisyErrorMessage(err);
      if (text) setMessage({ ok: false, text });
    } finally {
      setBusy(false);
    }
  };

  return (
    <Card sx={{ height: '100%' }}>
      <CardContent sx={{ p: 3 }}>
      <Typography variant="h6" gutterBottom>
        Top up from OISY
      </Typography>
      <Typography variant="body2" color="text.secondary" sx={{ mb: 2 }}>
        Move ICP or $CHRONO from your OISY wallet into your Chronolock wallet. You
        approve the transfer in OISY.
      </Typography>
      {isLocal ? (
        <Alert severity="info">
          OISY runs on mainnet only. On a local network, fund this wallet with the
          commands in the README instead.
        </Alert>
      ) : !accounts || accounts.length === 0 ? (
        <Button variant="contained" onClick={connect}>
          Connect OISY
        </Button>
      ) : (
        <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
          {accounts.length > 1 && (
            <TextField select label="OISY account" value={selected} onChange={(e) => setSelected(e.target.value)}>
              {accounts.map((a) => (
                <MenuItem key={encodeIcrcAccount(a)} value={encodeIcrcAccount(a)}>
                  {encodeIcrcAccount(a)}
                </MenuItem>
              ))}
            </TextField>
          )}
          <ToggleButtonGroup exclusive size="small" value={ledgerId} onChange={(_, v) => v && setLedgerId(v)}>
            {tokens.map((t) => (
              <ToggleButton key={t.ledger.toText()} value={t.ledger.toText()}>
                {t.symbol}
              </ToggleButton>
            ))}
          </ToggleButtonGroup>
          {token && (
            <Typography variant="body2">
              OISY balance: {oisyBal === undefined ? '…' : `${formatUnits(oisyBal, token.decimals)} ${token.symbol}`}
            </Typography>
          )}
          <TextField
            label={`Amount (${token?.symbol ?? ''})`}
            value={amount}
            onChange={(e) => setAmount(e.target.value)}
            inputMode="decimal"
          />
          <Box sx={{ display: 'flex', gap: 1 }}>
            <Button variant="contained" onClick={topUp} disabled={busy || !amount}>
              {busy ? 'Waiting for OISY…' : 'Top up'}
            </Button>
            <Button
              onClick={() => {
                forgetOisy();
                setAccounts(null);
                setSelected('');
              }}
            >
              Disconnect
            </Button>
          </Box>
        </Box>
      )}
      {message && (
        <Alert severity={message.ok ? 'success' : 'error'} sx={{ mt: 2 }}>
          {message.text}
        </Alert>
      )}
      </CardContent>
    </Card>
  );
};
