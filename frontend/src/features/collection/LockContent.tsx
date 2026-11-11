import DownloadIcon from '@mui/icons-material/Download';
import { Box, Button, Card, CardContent, Typography } from '@mui/material';
import type { DecryptedLock } from '../../api/backend';

/** Renders decrypted chronolock content: text plus an image, video, audio or file. */
export const LockContent = ({ data }: { data: DecryptedLock }) => {
  const type = data.fileType ?? '';
  return (
    <Card>
      <CardContent sx={{ p: { xs: 3, md: 4 } }}>
        <Typography variant="h4" gutterBottom>
          {data.title}
        </Typography>
        {data.description && (
          <Typography sx={{ whiteSpace: 'pre-wrap', lineHeight: 1.8 }}>{data.description}</Typography>
        )}
        {data.attributes && data.attributes.length > 0 && (
          <Box sx={{ mt: 3, display: 'grid', gridTemplateColumns: 'auto 1fr', gap: 1 }}>
            {data.attributes.map((a) => (
              <Box key={a.key} sx={{ display: 'contents' }}>
                <Typography color="text.secondary">{a.key}</Typography>
                <Typography>{a.value}</Typography>
              </Box>
            ))}
          </Box>
        )}
        {data.mediaUrl && (
          <Box sx={{ mt: 3 }}>
            {type.startsWith('image/') && (
              <Box component="img" src={data.mediaUrl} alt={data.fileName ?? 'Attachment'} sx={{ maxWidth: '100%', borderRadius: 2 }} />
            )}
            {type.startsWith('video/') && (
              <Box component="video" src={data.mediaUrl} controls sx={{ width: '100%', borderRadius: 2 }} />
            )}
            {type.startsWith('audio/') && <audio src={data.mediaUrl} controls style={{ width: '100%' }} />}
            <Button
              sx={{ mt: 2 }}
              variant="outlined"
              startIcon={<DownloadIcon />}
              href={data.mediaUrl}
              download={data.fileName ?? 'chronolock-file'}
            >
              Download {data.fileName ?? 'file'}
            </Button>
          </Box>
        )}
      </CardContent>
    </Card>
  );
};
