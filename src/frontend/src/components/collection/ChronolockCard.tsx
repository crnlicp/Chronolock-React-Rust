import React, { useState } from 'react';
import { NavLink } from 'react-router';
import {
  Box,
  Typography,
  Card,
  CardContent,
  CardActions,
  Button,
  Chip,
} from '@mui/material';
import { Chronolock, useChronolock } from '../../hooks/useChronolock';
import Clock from '../Clock';
import { useAuth } from '../../hooks/useAuth';
import { DecryptModal } from './DecryptModal';
import { useLockTimer } from '../../hooks/useLockTimer';

interface ChronolockCardProps {
  chronolock: Chronolock;
  onDelete?: () => void;
}

export const ChronolockCard: React.FC<ChronolockCardProps> = ({
  chronolock,
  onDelete,
}) => {
  const [decryptModalOpen, setDecryptModalOpen] = useState(false);
  const { principal } = useAuth();
  const { burnChronolock, isBurnChronolockLoading } = useChronolock();

  // Convert bigint to number for unlock_time if needed
  const unlockTime =
    typeof chronolock.unlock_time === 'bigint'
      ? Number(chronolock.unlock_time)
      : chronolock.unlock_time;

  const createdAt =
    typeof chronolock.created_at === 'bigint'
      ? Number(chronolock.created_at)
      : chronolock.created_at;

  // Use the lock timer hook for efficient real-time updates
  const isLocked = useLockTimer(unlockTime, chronolock.id);

  const isPublic = chronolock.user_keys?.[0]?.user === 'public';

  const isDecryptable = chronolock.user_keys
    ? chronolock.user_keys.some((uk) => uk.user === principal) || isPublic
    : false;

  const ownerStr = (() => {
    const ownerText =
      typeof chronolock.owner === 'string'
        ? chronolock.owner
        : chronolock.owner.toText();
    return `${ownerText.slice(0, 5)}...${ownerText.slice(-3)}`;
  })();

  const handleClickDecrypt = () => {
    if (!isDecryptable) return;
    setDecryptModalOpen(true);
  };

  const handleBurn = async () => {
    if (
      !window.confirm(
        'Are you sure you want to burn this Chronolock? This action cannot be undone.',
      )
    ) {
      return;
    }
    try {
      await burnChronolock(chronolock.id);
      if (onDelete) {
        onDelete();
      }
    } catch (error) {
      console.error('Error burning chronolock:', error);
    }
  };

  const isOwner =
    principal ===
    (typeof chronolock.owner === 'string'
      ? chronolock.owner
      : chronolock.owner.toText());

  return (
    <Card
      sx={{
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
        backgroundColor: '#f9f9f9',
      }}
    >
      <CardContent sx={{ flexGrow: 1 }}>
        <Typography variant="h6" component="div" gutterBottom>
          <NavLink
            to={`/chronolock/${chronolock.id}`}
            style={{ textDecoration: 'none', color: 'inherit' }}
          >
            {chronolock.title
              ? chronolock.title.slice(0, 19)
              : `Chronolock #${chronolock.id.slice(0, 8)}...`}
          </NavLink>
        </Typography>

        <Typography variant="body2" color="text.secondary" gutterBottom>
          <strong>Owner:</strong> {ownerStr.slice(0, 20)}
        </Typography>

        <Typography variant="body2" color="text.secondary" gutterBottom>
          <strong>Created date:</strong>{' '}
          {createdAt ? new Date(createdAt).toLocaleString() : 'Unknown'}
        </Typography>

        <Box sx={{ mt: 2 }}>
          <Chip
            label={isLocked ? 'Locked' : 'Unlocked'}
            color={isLocked ? 'error' : 'success'}
            size="small"
            onClick={() => {}}
            clickable={false}
          />
          {chronolock.user_keys && chronolock.user_keys.length > 0 && (
            <Chip
              label={
                isPublic
                  ? 'Public'
                  : isDecryptable
                  ? chronolock.user_keys.length > 1
                    ? `You + ${chronolock.user_keys.length - 1} recipient(s)`
                    : 'Only You'
                  : `${chronolock.user_keys.length} recipient(s)`
              }
              variant="outlined"
              size="small"
              sx={{ ml: 1 }}
              onClick={() => {}}
              clickable={false}
            />
          )}
          {isOwner && (
            <Chip
              label="U’re owner"
              variant="outlined"
              color="primary"
              size="small"
              sx={{ ml: 1 }}
              onClick={() => {}}
              clickable={false}
            />
          )}
        </Box>
      </CardContent>

      <Box sx={{ position: 'relative', width: '100%', height: '100%' }}>
        <img
          src={
            !isDecryptable || isLocked
              ? `/assets/img/lock.png`
              : `/assets/img/unlocked.png`
          }
          width={'100%'}
          height={'100%'}
        />
        <Clock targetDate={new Date(unlockTime * 1000)} className="abs_img" />
      </Box>
      {principal ? (
        <CardActions>
          <Button
            size="small"
            variant="outlined"
            color={'success'}
            disabled={!isDecryptable || isLocked}
            fullWidth
            onClick={handleClickDecrypt}
          >
            {!isDecryptable ? 'Not for you' : isLocked ? 'Locked' : 'Decrypt'}
          </Button>
          {isOwner && (
            <Button
              size="small"
              variant="outlined"
              color="error"
              fullWidth
              onClick={handleBurn}
              disabled={isBurnChronolockLoading}
            >
              {isBurnChronolockLoading ? 'Burning...' : 'Burn'}
            </Button>
          )}
        </CardActions>
      ) : (
        <Box
          sx={{
            textAlign: 'center',
            border: '1px solid #ccc',
            borderRadius: 1,
            p: 1,
            m: 1,
          }}
        >
          <Typography variant="body2" color="text.secondary" sx={{ p: 1 }}>
            Login to interact.
          </Typography>
        </Box>
      )}

      {/* Decrypt Modal */}
      <DecryptModal
        open={decryptModalOpen}
        onClose={() => setDecryptModalOpen(false)}
        chronolock={chronolock}
      />
    </Card>
  );
};
