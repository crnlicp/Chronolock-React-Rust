import React, { useState, useEffect } from 'react';
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

interface ChronolockMetadata {
  title?: string;
  owner?: string;
  lockTime?: number;
  createdAt?: number;
  userKeys?: { user: string; key: string }[];
  encryptedMetaData?: string;
}

interface ChronolockCardProps {
  chronolock: Chronolock;
  onDelete?: () => void;
}

export const ChronolockCard: React.FC<ChronolockCardProps> = ({
  chronolock,
  onDelete,
}) => {
  const [metadata, setMetadata] = useState<ChronolockMetadata | null>(null);
  const [decryptModalOpen, setDecryptModalOpen] = useState(false);
  const { principal } = useAuth();
  const { burnChronolock, isBurnChronolockLoading } = useChronolock();

  // Use the lock timer hook for efficient real-time updates
  const isLocked = useLockTimer(metadata?.lockTime, chronolock.id);

  useEffect(() => {
    try {
      // Decode base64 metadata
      const decodedMetadata = atob(chronolock.metadata);
      const metadataObj = JSON.parse(decodedMetadata);
      setMetadata(metadataObj);
    } catch (error) {
      console.error('Error parsing chronolock metadata:', error);
    }
  }, [chronolock.metadata]);

  const isPublic = metadata?.userKeys?.[0]?.user === 'public';

  const isDecryptable = metadata?.userKeys
    ? metadata.userKeys.some((uk) => uk.user === principal) || isPublic
    : false;

  const ownerStr = (() => {
    if (typeof metadata?.owner === 'string') {
      return `${metadata?.owner.slice(0, 5)}...${metadata?.owner.slice(-3)}`;
    } else {
      return `${String(metadata?.owner).slice(0, 5)}...${String(
        metadata?.owner,
      ).slice(-3)}`;
    }
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

  const isOwner = principal === metadata?.owner?.toString();

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
            {metadata?.title
              ? metadata?.title?.slice(0, 19)
              : `Chronolock #${chronolock.id.slice(0, 8)}...`}
          </NavLink>
        </Typography>

        <Typography variant="body2" color="text.secondary" gutterBottom>
          <strong>Owner:</strong> {ownerStr.slice(0, 20)}
        </Typography>

        <Typography variant="body2" color="text.secondary" gutterBottom>
          <strong>Created date:</strong>{' '}
          {metadata?.createdAt
            ? new Date(metadata.createdAt).toLocaleString()
            : 'Unknown'}
        </Typography>

        <Box sx={{ mt: 2 }}>
          <Chip
            label={isLocked ? 'Locked' : 'Unlocked'}
            color={isLocked ? 'error' : 'success'}
            size="small"
            onClick={() => {}}
            clickable={false}
          />
          {metadata?.userKeys && metadata.userKeys.length > 0 && (
            <Chip
              label={
                isPublic
                  ? 'Public'
                  : isDecryptable
                  ? metadata.userKeys.length > 1
                    ? `You + ${metadata.userKeys.length - 1} recipient(s)`
                    : 'Only You'
                  : `${metadata.userKeys.length} recipient(s)`
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
              label="You are the owner"
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
        <Clock
          targetDate={new Date((metadata?.lockTime ?? 0) * 1000)}
          className="abs_img"
        />
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
            {!isDecryptable
              ? 'Not available to you'
              : isLocked
              ? 'Locked'
              : 'Decrypt'}
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
      {metadata && (
        <DecryptModal
          open={decryptModalOpen}
          onClose={() => setDecryptModalOpen(false)}
          metadata={metadata}
        />
      )}
    </Card>
  );
};
