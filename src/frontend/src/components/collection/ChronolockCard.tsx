import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  Card,
  CardContent,
  CardActions,
  Button,
  Chip,
} from '@mui/material';
import { Chronolock } from '../../hooks/useChronolock';
import Clock from '../Clock';
import { useAuth } from '../../hooks/useAuth';
import { DecryptModal } from './DecryptModal';

interface ChronolockMetadata {
  title?: string;
  lockTime?: number;
  userKeys?: { user: string; key: string }[];
  encryptedMetaData?: string;
}

interface ChronolockCardProps {
  chronolock: Chronolock;
}

export const ChronolockCard: React.FC<ChronolockCardProps> = ({
  chronolock,
}) => {
  const [metadata, setMetadata] = useState<ChronolockMetadata | null>(null);
  const [decryptModalOpen, setDecryptModalOpen] = useState(false);
  const { principal } = useAuth();

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

  const isLocked = metadata?.lockTime
    ? Date.now() / 1000 < metadata.lockTime
    : true;

  const isPublic = metadata?.userKeys?.[0]?.user === 'public';

  const isDecryptable = metadata?.userKeys
    ? (metadata.userKeys.some((uk) => uk.user === principal) || isPublic) &&
      !isLocked
    : false;

  const ownerStr = (() => {
    if (typeof chronolock.owner === 'string') {
      return `${chronolock.owner.slice(0, 5)}...${chronolock.owner.slice(-3)}`;
    } else {
      return `${String(chronolock.owner).slice(0, 5)}...${String(
        chronolock.owner,
      ).slice(-3)}`;
    }
  })();

  const handleClickDecrypt = () => {
    if (!isDecryptable) return;
    setDecryptModalOpen(true);
  };
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
          {metadata?.title?.slice(0, 17) + '...' ||
            `Chronolock #${chronolock.id.slice(0, 8)}...`}
        </Typography>

        <Typography variant="body2" color="text.secondary" gutterBottom>
          <strong>Owner:</strong> {ownerStr.slice(0, 20)}
        </Typography>

        <Box sx={{ mt: 2 }}>
          <Chip
            label={isLocked ? 'Locked' : 'Unlocked'}
            color={isLocked ? 'error' : 'success'}
            size="small"
          />
          {metadata?.userKeys && metadata.userKeys.length > 0 && (
            <Chip
              label={
                isPublic ? 'public' : `${metadata.userKeys.length} recipient(s)`
              }
              variant="outlined"
              size="small"
              sx={{ ml: 1 }}
            />
          )}
        </Box>
      </CardContent>

      <Box sx={{ position: 'relative', width: '100%', height: '100%' }}>
        <img src="assets/img/lock.png" width={'100%'} height={'100%'} />
        <Clock
          targetDate={new Date((metadata?.lockTime ?? 0) * 1000)}
          className="abs_img"
        />
      </Box>
      {principal ? (
        <CardActions sx={{ padding: 1 }}>
          <Button
            size="small"
            variant="outlined"
            color={'success'}
            disabled={!isDecryptable}
            fullWidth
            onClick={handleClickDecrypt}
          >
            {!isDecryptable ? 'Locked for you' : 'Decrypt'}
          </Button>
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
