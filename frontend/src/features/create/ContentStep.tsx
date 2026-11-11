import CloudUploadIcon from '@mui/icons-material/CloudUpload';
import DeleteIcon from '@mui/icons-material/Delete';
import InsertDriveFileIcon from '@mui/icons-material/InsertDriveFile';
import {
  Alert,
  Box,
  Button,
  IconButton,
  LinearProgress,
  Stack,
  TextField,
  Typography,
} from '@mui/material';
import { alpha } from '@mui/material/styles';
import { useEffect, useMemo, useState } from 'react';
import { useDropzone } from 'react-dropzone';
import { uploadMedia, useBackend } from '../../api/backend';
import { AES_OVERHEAD } from '../../lib/crypto';
import { errorMessage } from '../../lib/errors';
import type { Draft } from './types';

const TITLE_MAX = 120;

interface Props {
  draft: Draft;
  aesKey: CryptoKey | null;
  maxMediaBytes: number;
  chunkBytes: number;
  onChange: (patch: Partial<Draft>) => void;
  onBack: () => void;
  onNext: () => void;
}

function formatBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  if (n < 1024 * 1024) return `${(n / 1024).toFixed(1)} KB`;
  return `${(n / 1024 / 1024).toFixed(1)} MB`;
}

export const ContentStep = ({ draft, aesKey, maxMediaBytes, chunkBytes, onChange, onBack, onNext }: Props) => {
  const backend = useBackend();
  const [progress, setProgress] = useState<number | null>(null);
  const [error, setError] = useState<string | null>(null);
  const maxFile = maxMediaBytes - Math.ceil(maxMediaBytes / chunkBytes) * AES_OVERHEAD;
  const preview = useMemo(
    () => (draft.file && /^(image|video)\//.test(draft.file.type) ? URL.createObjectURL(draft.file) : null),
    [draft.file],
  );
  useEffect(() => () => {
    if (preview) URL.revokeObjectURL(preview);
  }, [preview]);

  const removeFile = () => {
    if (draft.mediaId) void backend.cancel_media_upload(BigInt(draft.mediaId)).catch(() => undefined);
    onChange({ file: null, mediaId: null });
  };

  const { getRootProps, getInputProps, isDragActive } = useDropzone({
    multiple: false,
    maxSize: maxFile,
    disabled: progress !== null,
    onDrop: (accepted, rejected) => {
      setError(rejected[0] ? `File too large. The limit is ${formatBytes(maxFile)}.` : null);
      if (accepted[0]) {
        if (draft.mediaId) void backend.cancel_media_upload(BigInt(draft.mediaId)).catch(() => undefined);
        onChange({ file: accepted[0], mediaId: null });
      }
    },
  });

  const next = async () => {
    setError(null);
    if (draft.file && !draft.mediaId) {
      if (!aesKey) return;
      setProgress(0);
      try {
        const data = new Uint8Array(await draft.file.arrayBuffer());
        const id = await uploadMedia(backend, aesKey, data, chunkBytes, (d, t) =>
          setProgress(Math.round((d / t) * 100)),
        );
        onChange({ mediaId: id.toString() });
      } catch (err) {
        setError(errorMessage(err));
        return;
      } finally {
        setProgress(null);
      }
    }
    onNext();
  };

  const uploading = progress !== null;

  return (
    <Stack spacing={3}>
      <TextField
        label="Title"
        value={draft.title}
        onChange={(e) => onChange({ title: e.target.value.slice(0, TITLE_MAX) })}
        helperText={`Encrypted like everything else. ${draft.title.length}/${TITLE_MAX}`}
        required
      />
      <TextField
        label="Message"
        value={draft.message}
        onChange={(e) => onChange({ message: e.target.value })}
        multiline
        minRows={6}
        placeholder="Write something your future reader will love…"
      />

      <Box>
        <Typography variant="h6" gutterBottom>
          Attach a file <Typography component="span" color="text.secondary">(optional)</Typography>
        </Typography>
        {draft.file ? (
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, p: 2, border: 1, borderColor: 'divider', borderRadius: 2 }}>
            {preview && draft.file.type.startsWith('image/') ? (
              <Box component="img" src={preview} alt="" sx={{ width: 72, height: 72, objectFit: 'cover', borderRadius: 1 }} />
            ) : preview ? (
              <Box component="video" src={preview} sx={{ width: 96, height: 72, borderRadius: 1 }} />
            ) : (
              <InsertDriveFileIcon sx={{ fontSize: 48, color: 'text.secondary' }} />
            )}
            <Box sx={{ flexGrow: 1, minWidth: 0 }}>
              <Typography noWrap>{draft.file.name}</Typography>
              <Typography variant="body2" color="text.secondary">
                {formatBytes(draft.file.size)} · {draft.mediaId ? 'encrypted and uploaded' : 'encrypted when you continue'}
              </Typography>
            </Box>
            <IconButton onClick={removeFile} disabled={uploading} aria-label="Remove file">
              <DeleteIcon />
            </IconButton>
          </Box>
        ) : (
          <Box
            {...getRootProps()}
            sx={{
              border: 2,
              borderStyle: 'dashed',
              borderColor: isDragActive ? 'primary.main' : 'divider',
              borderRadius: 3,
              p: 5,
              textAlign: 'center',
              cursor: 'pointer',
              bgcolor: (t) => (isDragActive ? alpha(t.palette.primary.main, 0.08) : 'transparent'),
            }}
          >
            <input {...getInputProps()} />
            <CloudUploadIcon sx={{ fontSize: 40, color: 'text.secondary' }} />
            <Typography>Drop a photo, video or file here, or click to choose</Typography>
            <Typography variant="body2" color="text.secondary">
              Up to {formatBytes(maxFile)}. Adding a file makes it a media chronolock.
            </Typography>
          </Box>
        )}
        {uploading && (
          <Box sx={{ mt: 2 }}>
            <Typography variant="body2" color="text.secondary" gutterBottom>
              Encrypting and uploading… {progress}%
            </Typography>
            <LinearProgress variant="determinate" value={progress ?? 0} />
          </Box>
        )}
      </Box>

      {error && <Alert severity="error">{error}</Alert>}

      <Box sx={{ display: 'flex', justifyContent: 'space-between' }}>
        <Button onClick={onBack} disabled={uploading}>
          Back
        </Button>
        <Button variant="contained" size="large" onClick={next} disabled={!draft.title.trim() || uploading || !aesKey}>
          {draft.file && !draft.mediaId ? 'Encrypt & continue' : 'Next'}
        </Button>
      </Box>
    </Stack>
  );
};
