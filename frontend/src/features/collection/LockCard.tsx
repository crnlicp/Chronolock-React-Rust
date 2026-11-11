import LockIcon from '@mui/icons-material/Lock';
import LockOpenIcon from '@mui/icons-material/LockOpen';
import { Box, Card, CardActionArea, CardContent, Chip, Stack, Typography } from '@mui/material';
import dayjs from 'dayjs';
import { Link as RouterLink } from 'react-router';
import type { Chronolock } from '../../bindings/backend';
import { useAuth } from '../../auth/AuthProvider';
import { ClockLock } from '../../components/clock-lock/ClockLock';
import { useLockTimer } from '../../hooks/useLockTimer';
import { viewOf } from './status';

export const LockCard = ({ lock }: { lock: Chronolock }) => {
  const { principal } = useAuth();
  const unlockAt = Number(lock.unlock_time);
  const locked = useLockTimer(unlockAt, lock.id.toString());
  const v = viewOf(lock, principal);

  return (
    <Card sx={{ height: '100%', transition: 'transform .2s, border-color .2s', '&:hover': { transform: 'translateY(-3px)', borderColor: 'primary.main' } }}>
      <CardActionArea component={RouterLink} to={`/chronolock/${lock.id}`} sx={{ height: '100%', display: 'flex', flexDirection: 'column', alignItems: 'stretch' }}>
        <Box sx={{ p: 3, pb: 0, display: 'flex', justifyContent: 'center' }}>
          <ClockLock target={new Date(unlockAt * 1000)} unlocked={!locked} size={200} />
        </Box>
        <CardContent sx={{ flexGrow: 1 }}>
          <Stack direction="row" sx={{ justifyContent: 'space-between', alignItems: 'center', gap: 1, mb: 1 }}>
            <Typography variant="subtitle1" noWrap sx={{ fontWeight: 700 }}>
              Chronolock #{lock.id.toString()}
            </Typography>
            <Chip
              size="small"
              icon={locked ? <LockIcon /> : <LockOpenIcon />}
              label={locked ? 'Locked' : 'Unlocked'}
              color={locked ? 'default' : 'success'}
            />
          </Stack>
          <Typography variant="body2" color="text.secondary">
            {locked ? 'Opens' : 'Opened'} {dayjs.unix(unlockAt).format('D MMM YYYY, HH:mm')}
          </Typography>
          <Stack direction="row" sx={{ flexWrap: 'wrap', gap: 0.75, mt: 1.5 }}>
            {v.isPublic ? (
              <Chip size="small" variant="outlined" label="Public" />
            ) : (
              <Chip size="small" variant="outlined" label={`${v.recipientCount} recipient${v.recipientCount === 1 ? '' : 's'}`} />
            )}
            {lock.media && <Chip size="small" variant="outlined" label="Media" />}
            {v.isOwner && <Chip size="small" color="primary" variant="outlined" label="Yours" />}
            {!v.isPublic && v.canOpen && !v.isOwner && <Chip size="small" color="secondary" variant="outlined" label="For you" />}
          </Stack>
        </CardContent>
      </CardActionArea>
    </Card>
  );
};
