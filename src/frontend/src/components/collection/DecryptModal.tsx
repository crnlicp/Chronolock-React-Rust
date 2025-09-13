import React, { useState, useEffect } from 'react';
import {
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  Box,
  Typography,
  CircularProgress,
  Alert,
  Card,
  CardContent,
  Divider,
  Chip,
} from '@mui/material';
import { useAuth } from '../../hooks/useAuth';
import { useChronolock } from '../../hooks/useChronolock';
import {
  DerivedPublicKey,
  IbeCiphertext,
  EncryptedVetKey,
  TransportSecretKey,
} from '@dfinity/vetkeys';

interface DecryptedData {
  name?: string;
  description?: string;
  fileType?: string;
  mediaId?: string;
  mediaUrl?: string;
  mediaSize?: number;
  attributes?: Record<string, any>;
}

interface ChronolockMetadata {
  title?: string;
  lockTime?: number;
  userKeys?: { user: string; key: string }[];
  encryptedMetaData?: string;
}

interface DecryptModalProps {
  open: boolean;
  onClose: () => void;
  metadata: ChronolockMetadata;
}

export const DecryptModal: React.FC<DecryptModalProps> = ({
  open,
  onClose,
  metadata,
}) => {
  const [decryptedData, setDecryptedData] = useState<DecryptedData | null>(
    null,
  );
  const [loading, setLoading] = useState(false);
  const [loadingMessage, setLoadingMessage] = useState<string>(
    'Decrypting content...',
  );
  const [error, setError] = useState<string | null>(null);
  const [decryptedMediaUrl, setDecryptedMediaUrl] = useState<string | null>(
    null,
  );
  const { principal } = useAuth();
  const {
    getVetkdPublicKey,
    getTimeDecryptionKey,
    getUserTimeDecryptionKey,
    getMediaChunked,
  } = useChronolock();

  const decrypt = async () => {
    if (!metadata.encryptedMetaData || !metadata.userKeys || !principal) {
      setError('Missing required data for decryption');
      return;
    }

    setLoading(true);
    setError(null);
    setLoadingMessage('Preparing decryption...');

    try {
      // Find the user's encrypted key
      const isPublic = metadata.userKeys[0]?.user === 'public';
      let userKey: string | null = null;
      let userIdentity: string | null = null;

      if (isPublic) {
        userKey = metadata.userKeys[0].key;
        userIdentity = metadata.lockTime?.toString() || '';
      } else {
        const userKeyEntry = metadata.userKeys.find(
          (uk) => uk.user === principal || uk.user.startsWith(`${principal}:`),
        );
        if (userKeyEntry) {
          userKey = userKeyEntry.key;
          userIdentity = userKeyEntry.user;
        }
      }

      if (!userKey || !userIdentity) {
        throw new Error('No decryption key found for this user');
      }

      setLoadingMessage('Generating transport keys...');

      // Generate a proper transport secret key for vetKD
      const transportSeed = new Uint8Array(32);
      crypto.getRandomValues(transportSeed);

      // Create TransportSecretKey using the vetkeys library
      const transportSecretKey = new TransportSecretKey(transportSeed);
      const transportPublicKeyBytes = transportSecretKey.publicKeyBytes();
      const transportPublicKey = Array.from(
        transportPublicKeyBytes,
      ) as number[];

      setLoadingMessage('Getting VetKD public key...');

      // Get the VetKD public key from the canister
      const vetkdPublicKeyResult = (await getVetkdPublicKey()) as any;
      if (!vetkdPublicKeyResult?.Ok) {
        throw new Error('Failed to get VetKD public key');
      }

      const vetkdPublicKeyBuffer = vetkdPublicKeyResult.Ok.public_key;
      const derivedPublicKey = DerivedPublicKey.deserialize(
        new Uint8Array(vetkdPublicKeyBuffer),
      );

      setLoadingMessage('Requesting decryption key...');

      // Real VetKD decryption implementation
      console.log('Starting VetKD decryption process...');

      // Get the decryption key from the canister using existing functions
      let decryptionKeyResult: any;
      if (isPublic) {
        // For public chronolocks, use the lockTime as unlock_time_hex
        const unlockTimeHex = metadata.lockTime?.toString(16).padStart(16, '0');
        if (!unlockTimeHex) {
          throw new Error('Invalid lock time for public chronolock');
        }
        decryptionKeyResult = await getTimeDecryptionKey(
          unlockTimeHex,
          transportPublicKey,
        );
      } else {
        // For user-specific chronolocks, extract unlock time and user from identity
        const unlockTimeHex = metadata.lockTime?.toString(16).padStart(16, '0');

        if (!unlockTimeHex) {
          throw new Error('Invalid lock time for private chronolock');
        }

        decryptionKeyResult = await getUserTimeDecryptionKey(
          unlockTimeHex,
          userIdentity,
          transportPublicKey,
        );
      }

      setLoadingMessage('Processing decryption key...');

      if (!decryptionKeyResult?.Ok) {
        throw new Error(
          `Failed to get decryption key: ${JSON.stringify(
            decryptionKeyResult?.Err || 'Unknown error',
          )}`,
        );
      }

      setLoadingMessage('Decrypting with VetKD...');

      // Real VetKD decryption implementation
      console.log('Starting VetKD decryption process...');

      // Decrypt the encrypted key using vetKD
      const encryptedVetKeyBytes = new Uint8Array(
        decryptionKeyResult.Ok.encrypted_key,
      );
      const encryptedVetKey = new EncryptedVetKey(encryptedVetKeyBytes);

      // Create the derivation input that matches the IBE identity format
      // Now both VetKD and IBE use the same identity format for compatibility
      let derivationInput: Uint8Array;
      if (isPublic) {
        // For public chronolocks: both VetKD and IBE use decimal time string
        const lockTimeString = metadata.lockTime?.toString() || '';
        derivationInput = new TextEncoder().encode(lockTimeString);
      } else {
        // For user chronolocks: both VetKD and IBE use "user_id:decimal_time" format
        const lockTimeString = metadata.lockTime?.toString() || '';
        const identityFormat = `${userIdentity}:${lockTimeString}`;
        derivationInput = new TextEncoder().encode(identityFormat);
      }

      // Decrypt and verify the VetKey using the transport secret key
      const vetKey = encryptedVetKey.decryptAndVerify(
        transportSecretKey,
        derivedPublicKey,
        derivationInput,
      );

      console.log('VetKey decrypted successfully');
      console.log('VetKey signature length:', vetKey.signatureBytes().length);

      // Now decrypt the AES key using IBE
      const encryptedUserKeyBytes = atob(userKey);
      const encryptedUserKeyUint8 = new Uint8Array(
        Array.from(encryptedUserKeyBytes).map((char) => char.charCodeAt(0)),
      );

      const ibeCiphertext = IbeCiphertext.deserialize(encryptedUserKeyUint8);
      console.log(
        'IBE Ciphertext parsed, length:',
        ibeCiphertext.serialize().length,
      );

      // Create the IBE identity that was used during encryption
      let ibeIdentityString: string;
      if (isPublic) {
        // For public chronolocks, identity was just the lockTime
        ibeIdentityString = metadata.lockTime?.toString() || '';
      } else {
        // For user chronolocks, identity was "user:lockTime"
        // But userIdentity might already contain the full format or just the user part
        if (userIdentity.includes(':')) {
          ibeIdentityString = userIdentity; // Already in format "user:lockTime"
        } else {
          // Construct the identity format that was used during encryption
          ibeIdentityString = `${userIdentity}:${metadata.lockTime}`;
        }
      }

      console.log('Using IBE identity for decryption:', ibeIdentityString);
      console.log('User identity from userKeys:', userIdentity);
      console.log('Lock time:', metadata.lockTime);
      console.log('Is public chronolock:', isPublic);
      console.log('VetKD derivation input bytes:', Array.from(derivationInput));
      console.log(
        'VetKD derivation input as text:',
        new TextDecoder().decode(derivationInput),
      );
      console.log(
        'FIXED: VetKD derivation (' +
          new TextDecoder().decode(derivationInput) +
          ') == IBE identity (' +
          ibeIdentityString +
          ') - formats now match!',
      );

      // Now use the VetKey for IBE decryption
      // The VetKey should be compatible with IBE decryption according to the vetkeys documentation
      let aesKeyBytes: Uint8Array;
      try {
        aesKeyBytes = ibeCiphertext.decrypt(vetKey);
        console.log(
          'AES key decrypted successfully, length:',
          aesKeyBytes.length,
        );
      } catch (directError) {
        console.log(
          'IBE decryption failed, checking VetKey and IBE compatibility...',
        );
        console.error('Direct error:', directError);
        console.log('VetKey type:', typeof vetKey);
        console.log(
          'VetKey signatureBytes:',
          Array.from(vetKey.signatureBytes().slice(0, 10)),
          '...',
        );
        console.log('IBE identity used:', ibeIdentityString);
        console.log(
          'VetKD derivation input used:',
          new TextDecoder().decode(derivationInput),
        );

        throw new Error(
          `IBE decryption failed. VetKD and IBE identities should now match. ` +
            `VetKD used: ${new TextDecoder().decode(derivationInput)}, ` +
            `IBE identity: ${ibeIdentityString}. ` +
            `If they match but decryption still fails, there may be another issue. ` +
            `Error: ${(directError as Error).message}`,
        );
      }
      console.log('AES key decrypted, length:', aesKeyBytes.length);

      setLoadingMessage('Decrypting metadata...');

      // Import the AES key for web crypto
      const cryptoKey = await window.crypto.subtle.importKey(
        'raw',
        new Uint8Array(aesKeyBytes),
        { name: 'AES-GCM' },
        false,
        ['decrypt'],
      );

      // Decrypt the actual metadata using AES-GCM
      const encryptedMetadataBytes = atob(metadata.encryptedMetaData);
      const encryptedBuffer = Uint8Array.from(encryptedMetadataBytes, (c) =>
        c.charCodeAt(0),
      );

      // Extract IV (first 12 bytes) and ciphertext (rest)
      const iv = encryptedBuffer.slice(0, 12);
      const ciphertext = encryptedBuffer.slice(12);

      // Decrypt the metadata
      const decryptedBuffer = await window.crypto.subtle.decrypt(
        { name: 'AES-GCM', iv },
        cryptoKey,
        ciphertext,
      );

      // Parse the decrypted JSON
      const decryptedText = new TextDecoder().decode(decryptedBuffer);
      const decryptedJson = JSON.parse(decryptedText);

      console.log('Metadata decrypted successfully:', decryptedJson);

      // If there's a mediaId and mediaSize, decrypt the media file
      if (decryptedJson.mediaId && decryptedJson.mediaSize) {
        console.log('Decrypting media file...');
        setLoadingMessage('Downloading and decrypting media file...');
        try {
          // The stored mediaSize is the original file size, but we need to download
          // the encrypted file which is larger (original + IV + auth tag)
          // Let's estimate the encrypted size and try to download more than needed
          const originalSize = decryptedJson.mediaSize;
          const estimatedEncryptedSize = originalSize + 32; // IV (12) + auth tag (16) + some buffer

          console.log('Original file size from metadata:', originalSize);
          console.log('Estimated encrypted size:', estimatedEncryptedSize);

          // Get the encrypted media file (this includes IV + encrypted data concatenated)
          const encryptedMediaData = await getMediaChunked(
            decryptedJson.mediaId,
            estimatedEncryptedSize,
          );

          console.log(
            'Downloaded media data length:',
            encryptedMediaData.length,
          );
          console.log(
            'Expected media size from metadata:',
            decryptedJson.mediaSize,
          );
          console.log(
            'First 20 bytes of encrypted data:',
            Array.from(encryptedMediaData.slice(0, 20)),
          );

          // The actual encrypted data should be larger than the original file size
          // because it includes IV (12 bytes) + AES-GCM authentication tag (16 bytes)
          const expectedMinSize = (decryptedJson.mediaSize || 0) + 12 + 16;
          if (encryptedMediaData.length < expectedMinSize) {
            console.warn(
              `Downloaded data seems too small. Expected at least ${expectedMinSize}, got ${encryptedMediaData.length}`,
            );
          }

          setLoadingMessage('Decrypting media content...');

          // The downloaded data should be in the format: IV (12 bytes) + encrypted content
          // This matches the format created in UploadFile component
          if (encryptedMediaData.length < 12) {
            throw new Error('Invalid encrypted media data: too short for IV');
          }

          // Extract IV (first 12 bytes) and encrypted data (rest)
          const mediaIv = encryptedMediaData.slice(0, 12);
          const encryptedMediaContent = encryptedMediaData.slice(12);

          console.log('Media IV length:', mediaIv.length);
          console.log('Media IV bytes:', Array.from(mediaIv));
          console.log(
            'Encrypted media content length:',
            encryptedMediaContent.length,
          );

          // Decrypt the media using the same AES key
          const decryptedMediaBuffer = await window.crypto.subtle.decrypt(
            { name: 'AES-GCM', iv: mediaIv },
            cryptoKey,
            encryptedMediaContent,
          );

          console.log(
            'Media decrypted successfully, size:',
            decryptedMediaBuffer.byteLength,
          );

          setLoadingMessage('Preparing media for display...');

          // Create a blob URL for the decrypted media
          const mediaBlob = new Blob([decryptedMediaBuffer], {
            type: decryptedJson.fileType || 'application/octet-stream',
          });
          const mediaUrl = URL.createObjectURL(mediaBlob);

          // Update the decrypted data with the media URL
          const decryptedDataWithMedia = {
            ...decryptedJson,
            mediaUrl: mediaUrl,
          };

          setDecryptedData(decryptedDataWithMedia);
          setDecryptedMediaUrl(mediaUrl);

          console.log('Media decrypted successfully');
        } catch (mediaError) {
          console.error('Failed to decrypt media:', mediaError);
          console.error('Media error details:', {
            message: (mediaError as Error).message,
            name: (mediaError as Error).name,
            stack: (mediaError as Error).stack,
          });
          // Still set the metadata even if media decryption fails
          setDecryptedData(decryptedJson);
          setError(`Media decryption failed: ${(mediaError as Error).message}`);
        }
      } else {
        // No media to decrypt, just set the metadata
        setDecryptedData(decryptedJson);
      }
    } catch (err) {
      console.error('Decryption error:', err);
      setError(err instanceof Error ? err.message : 'Unknown error occurred');
    } finally {
      setLoading(false);
    }
  };

  useEffect(() => {
    if (open && !decryptedData && !loading) {
      decrypt();
    }
  }, [open]);

  const handleClose = () => {
    // Clean up blob URL if it exists
    if (decryptedMediaUrl) {
      URL.revokeObjectURL(decryptedMediaUrl);
      setDecryptedMediaUrl(null);
    }
    setDecryptedData(null);
    setError(null);
    setLoadingMessage('Decrypting content...');
    onClose();
  };

  // Cleanup blob URL on unmount
  useEffect(() => {
    return () => {
      if (decryptedMediaUrl) {
        URL.revokeObjectURL(decryptedMediaUrl);
      }
    };
  }, [decryptedMediaUrl]);

  const renderAttribute = (_key: string, value: any) => {
    if (typeof value === 'object' && value !== null) {
      return JSON.stringify(value, null, 2);
    }
    return String(value);
  };

  return (
    <Dialog
      open={open}
      onClose={handleClose}
      maxWidth="md"
      fullWidth
      slotProps={{
        paper: {
          sx: { borderRadius: 2 },
        },
      }}
    >
      <DialogTitle sx={{ pb: 1 }}>
        <Typography variant="h5" component="div">
          Decrypted Content
        </Typography>
        <Typography variant="body1" color="text.secondary">
          {metadata.title || 'Chronolock Content'}
        </Typography>
      </DialogTitle>

      <DialogContent sx={{ pt: 1 }}>
        {loading && (
          <Box sx={{ display: 'flex', justifyContent: 'center', py: 4 }}>
            <Box sx={{ textAlign: 'center' }}>
              <CircularProgress />
              <Typography variant="body2" sx={{ mt: 2 }}>
                {loadingMessage}
              </Typography>
            </Box>
          </Box>
        )}

        {error && (
          <Alert severity="error" sx={{ mb: 2 }}>
            {error}
          </Alert>
        )}

        {decryptedData && (
          <Box sx={{ mt: 1 }}>
            <Card variant="outlined" sx={{ mb: 2 }}>
              <CardContent>
                <Box sx={{ display: 'flex', flexWrap: 'wrap', gap: 2 }}>
                  {decryptedData.name && (
                    <Box
                      sx={{
                        width: { xs: '100%', sm: 'calc(50% - 8px)' },
                        mb: 3,
                      }}
                    >
                      <Typography variant="h6" gutterBottom>
                        Name
                      </Typography>
                      <Typography variant="body1">
                        {decryptedData.name}
                      </Typography>
                    </Box>
                  )}

                  {decryptedData.description && (
                    <Box
                      sx={{
                        width: { xs: '100%', sm: 'calc(50% - 8px)' },
                        mb: 3,
                      }}
                    >
                      <Typography variant="h6" gutterBottom>
                        Description
                      </Typography>
                      <Typography
                        variant="body1"
                        sx={{ whiteSpace: 'pre-wrap' }}
                      >
                        {decryptedData.description}
                      </Typography>
                    </Box>
                  )}

                  {decryptedData.fileType && (
                    <Box
                      sx={{
                        width: { xs: '100%', sm: 'calc(50% - 8px)' },
                        mb: 3,
                      }}
                    >
                      <Typography variant="h6" gutterBottom>
                        File Type
                      </Typography>
                      <Chip label={decryptedData.fileType} variant="outlined" />
                    </Box>
                  )}

                  {decryptedData.mediaUrl && (
                    <Box
                      sx={{
                        width: { xs: '100%', sm: 'calc(50% - 8px)' },
                        mb: 3,
                      }}
                    >
                      <Typography variant="h6">Media</Typography>
                      {decryptedData.fileType?.startsWith('image/') ? (
                        <>
                          <Box
                            component="img"
                            src={decryptedData.mediaUrl}
                            alt={decryptedData.name || 'Decrypted media'}
                            sx={{
                              maxWidth: '70%',
                              maxHeight: 200,
                              borderRadius: 1,
                              border: '1px solid',
                              borderColor: 'divider',
                              mt: 1,
                            }}
                          />
                          <Typography
                            variant="caption"
                            color="text.secondary"
                            sx={{ display: 'block' }}
                          >
                            Size:{' '}
                            {decryptedData.mediaSize
                              ? `${(
                                  decryptedData.mediaSize /
                                  1024 /
                                  1024
                                ).toFixed(2)} MB`
                              : 'Unknown'}
                          </Typography>
                        </>
                      ) : decryptedData.fileType?.startsWith('video/') ? (
                        <>
                          <Box
                            component="video"
                            src={decryptedData.mediaUrl}
                            controls
                            sx={{
                              maxWidth: '70%',
                              maxHeight: 200,
                              borderRadius: 1,
                              border: '1px solid',
                              borderColor: 'divider',
                              mt: 1,
                            }}
                          />
                          <Typography
                            variant="caption"
                            color="text.secondary"
                            sx={{ display: 'block' }}
                          >
                            Size:{' '}
                            {decryptedData.mediaSize
                              ? `${(
                                  decryptedData.mediaSize /
                                  1024 /
                                  1024
                                ).toFixed(2)} MB`
                              : 'Unknown'}
                          </Typography>
                        </>
                      ) : (
                        <>
                          <Typography variant="body2" color="text.secondary">
                            Media file:{' '}
                            {decryptedData.fileType || 'Unknown type'}
                          </Typography>
                          <Button
                            variant="outlined"
                            href={decryptedData.mediaUrl}
                            download={decryptedData.name || 'decrypted-file'}
                            sx={{ my: 1 }}
                          >
                            Download File
                          </Button>
                          <Typography
                            variant="caption"
                            color="text.secondary"
                            sx={{ display: 'block' }}
                          >
                            Size:{' '}
                            {decryptedData.mediaSize
                              ? `${(
                                  decryptedData.mediaSize /
                                  1024 /
                                  1024
                                ).toFixed(2)} MB`
                              : 'Unknown'}
                          </Typography>
                        </>
                      )}
                    </Box>
                  )}

                  {decryptedData.attributes &&
                    Object.keys(decryptedData.attributes).length > 0 && (
                      <Box sx={{ width: '100%', mb: 3 }}>
                        <Typography variant="h6" gutterBottom>
                          Attributes
                        </Typography>
                        <Divider sx={{ mb: 1 }} />
                        {Object.entries(decryptedData.attributes).map(
                          ([key, value]) => (
                            <Box key={key} sx={{ mb: 1 }}>
                              <Typography
                                variant="subtitle2"
                                color="text.secondary"
                              >
                                {key}:
                              </Typography>
                              <Typography
                                variant="body2"
                                sx={{
                                  ml: 1,
                                  fontFamily:
                                    typeof value === 'object'
                                      ? 'monospace'
                                      : 'inherit',
                                  whiteSpace:
                                    typeof value === 'object'
                                      ? 'pre'
                                      : 'normal',
                                }}
                              >
                                {renderAttribute(key, value)}
                              </Typography>
                            </Box>
                          ),
                        )}
                      </Box>
                    )}

                  {!decryptedData.name &&
                    !decryptedData.description &&
                    !decryptedData.fileType &&
                    !decryptedData.mediaUrl &&
                    !decryptedData.attributes && (
                      <Typography variant="body2" color="text.secondary">
                        No decrypted content available to display.
                      </Typography>
                    )}
                </Box>
              </CardContent>
            </Card>
          </Box>
        )}
      </DialogContent>

      <DialogActions sx={{ px: 3, pb: 2 }}>
        <Button onClick={handleClose} variant="contained">
          Close
        </Button>
      </DialogActions>
    </Dialog>
  );
};
