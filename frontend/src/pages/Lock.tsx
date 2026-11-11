import ArrowBackIcon from '@mui/icons-material/ArrowBack';
import DeleteIcon from '@mui/icons-material/Delete';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import {
  Alert,
  Box,
  Button,
  Card,
  CardContent,
  Chip,
  Dialog,
  DialogActions,
  DialogContent,
  DialogTitle,
  Grid,
  LinearProgress,
  Skeleton,
  Stack,
  Typography,
} from '@mui/material';
import dayjs from 'dayjs';
import { useEffect, useState } from 'react';
import { Link as RouterLink, useNavigate, useParams } from 'react-router';
import {
  decryptChronolock,
  ibePublicKeyFor,
  useBackend,
  useBurnChronolock,
  useChronolock,
  type DecryptedLock,
} from '../api/backend';
import { useAuth } from '../auth/AuthProvider';
import { ClockLock } from '../components/clock-lock/ClockLock';
import { CopyField, shortPrincipal } from '../components/ui/CopyField';
import { SignInPrompt } from '../components/ui/SignInPrompt';
import { LockContent } from '../features/collection/LockContent';
import { viewOf } from '../features/collection/status';
import { useLockTimer } from '../hooks/useLockTimer';
import { errorMessage } from '../lib/errors';
import { formatUnits } from '../lib/format';

function parseId(id?: string) {
  return id && /^\d+$/.test(id) ? BigInt(id) : undefined;
}

export const LockPage = () => {
  const { id } = useParams();
  const lockId = parseId(id);
  const { data: lock, isLoading } = useChronolock(lockId);
  const { principal, isAuthenticated } = useAuth();
  const backend = useBackend();
  const burn = useBurnChronolock();
  const navigate = useNavigate();
  const locked = useLockTimer(lock ? Number(lock.unlock_time) : undefined, id ?? '');
  const [content, setContent] = useState<DecryptedLock | null>(null);
  const [step, setStep] = useState<string | null>(null);
  const [error, setError] = useState<string | null>(null);
  const [confirmBurn, setConfirmBurn] = useState(false);

  useEffect(() => () => {
    if (content?.mediaUrl) URL.revokeObjectURL(content.mediaUrl);
  }, [content]);

  if (isLoading) return <Skeleton variant="rounded" height={480} />;
  if (!lock) {
    return (
      <Card sx={{ textAlign: 'center' }}>
        <CardContent sx={{ py: 8 }}>
          <Typography variant="h5" gutterBottom>
            Chronolock not found
          </Typography>
          <Button component={RouterLink} to="/collection">
            Back to collection
          </Button>
        </CardContent>
      </Card>
    );
  }

  const v = viewOf(lock, principal);
  const unlockAt = dayjs.unix(Number(lock.unlock_time));

  const open = async () => {
    if (!principal) return;
    setError(null);
    setStep('Preparing…');
    try {
      setContent(await decryptChronolock(backend, lock, principal, await ibePublicKeyFor(backend), setStep));
    } catch (err) {
      setError(errorMessage(err));
    } finally {
      setStep(null);
    }
  };

  const doBurn = async () => {
    try {
      await burn.mutateAsync(lock.id);
      navigate('/collection?tab=owned');
    } catch (err) {
      setError(errorMessage(err));
      setConfirmBurn(false);
    }
  };

  return (
    <>
      <Button component={RouterLink} to="/collection" startIcon={<ArrowBackIcon />} sx={{ mb: 3 }}>
        Collection
      </Button>
      <Grid container spacing={5}>
        <Grid size={{ xs: 12, md: 6 }} sx={{ display: 'flex', justifyContent: 'center' }}>
          <ClockLock target={unlockAt.toDate()} unlocked={!locked} />
        </Grid>
        <Grid size={{ xs: 12, md: 6 }}>
          <Stack direction="row" spacing={1} sx={{ mb: 2, flexWrap: 'wrap' }}>
            <Chip label={locked ? 'Locked' : 'Unlocked'} color={locked ? 'default' : 'success'} />
            <Chip variant="outlined" label={v.isPublic ? 'Public' : `${v.recipientCount} recipient${v.recipientCount === 1 ? '' : 's'}`} />
            {lock.media && <Chip variant="outlined" label="Media" />}
            {v.isOwner && <Chip variant="outlined" color="primary" label="Yours" />}
          </Stack>
          <Typography variant="h3" component="h1" gutterBottom>
            Chronolock #{lock.id.toString()}
          </Typography>
          <Typography color="text.secondary" sx={{ mb: 3 }}>
            {locked ? 'Opens' : 'Opened'} {unlockAt.format('dddd, D MMMM YYYY [at] HH:mm')}
          </Typography>

          {!isAuthenticated ? (
            <SignInPrompt inline message="Only its recipients can open it." />
          ) : content ? null : locked ? (
            <Alert severity="info">
              {v.canOpen
                ? 'You can open this when the countdown ends.'
                : "This chronolock wasn't shared with you."}
            </Alert>
          ) : v.canOpen ? (
            <Button variant="contained" size="large" startIcon={<LockOpenIcon />} onClick={open} disabled={!!step}>
              Open chronolock
            </Button>
          ) : (
            <Alert severity="warning">This chronolock wasn't shared with you.</Alert>
          )}
          {step && (
            <Box sx={{ mt: 2 }}>
              <Typography variant="body2" gutterBottom>
                {step}
              </Typography>
              <LinearProgress />
            </Box>
          )}
          {error && (
            <Alert severity="error" sx={{ mt: 2 }}>
              {error}
            </Alert>
          )}

          <Card sx={{ mt: 4 }}>
            <CardContent>
              <Grid container spacing={2}>
                <Grid size={{ xs: 12, sm: 6 }}>
                  <Typography variant="caption" color="text.secondary">
                    Owner
                  </Typography>
                  <Typography sx={{ fontFamily: 'monospace' }}>{shortPrincipal(lock.owner.toText())}</Typography>
                </Grid>
                <Grid size={{ xs: 12, sm: 6 }}>
                  <Typography variant="caption" color="text.secondary">
                    Sealed
                  </Typography>
                  <Typography>{dayjs(Number(lock.created_at)).format('D MMM YYYY, HH:mm')}</Typography>
                </Grid>
                {lock.payment && (
                  <Grid size={{ xs: 12, sm: 6 }}>
                    <Typography variant="caption" color="text.secondary">
                      Paid
                    </Typography>
                    <Typography>
                      {formatUnits(lock.payment.amount, 8)} {lock.payment.method}
                    </Typography>
                  </Grid>
                )}
              </Grid>
              <Box sx={{ mt: 2 }}>
                <CopyField label="Share link" value={window.location.href} />
              </Box>
              {v.isOwner && (
                <Button color="error" startIcon={<DeleteIcon />} onClick={() => setConfirmBurn(true)}>
                  Burn chronolock
                </Button>
              )}
            </CardContent>
          </Card>
        </Grid>
      </Grid>

      {content && (
        <Box sx={{ mt: 5 }}>
          <LockContent data={content} />
        </Box>
      )}

      <Dialog open={confirmBurn} onClose={() => setConfirmBurn(false)}>
        <DialogTitle>Burn chronolock #{lock.id.toString()}?</DialogTitle>
        <DialogContent>
          <Typography>
            This deletes it permanently, including any attached media. Recipients will not be able
            to open it. This cannot be undone.
          </Typography>
        </DialogContent>
        <DialogActions>
          <Button onClick={() => setConfirmBurn(false)}>Cancel</Button>
          <Button color="error" variant="contained" onClick={doBurn} disabled={burn.isPending}>
            Burn
          </Button>
        </DialogActions>
      </Dialog>
    </>
  );
};
