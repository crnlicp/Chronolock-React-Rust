import LoginIcon from '@mui/icons-material/Login';
import { Box, Button, Card, CardContent, Typography } from '@mui/material';
import { useAuth } from '../../auth/AuthProvider';

export const SignInPrompt = ({ message, inline }: { message: string; inline?: boolean }) => {
  const { signIn, isLoading } = useAuth();
  if (inline) {
    return (
      <Box sx={{ display: 'flex', flexWrap: 'wrap', alignItems: 'center', gap: 2 }}>
        <Button variant="contained" size="large" startIcon={<LoginIcon />} onClick={() => void signIn()} disabled={isLoading}>
          Sign in to open
        </Button>
        <Typography color="text.secondary">{message}</Typography>
      </Box>
    );
  }
  return (
    <Card sx={{ maxWidth: 480, mx: 'auto', my: 8, textAlign: 'center' }}>
      <CardContent sx={{ p: 4 }}>
        <Typography variant="h5" gutterBottom>
          Sign in to continue
        </Typography>
        <Typography color="text.secondary" sx={{ mb: 3 }}>
          {message}
        </Typography>
        <Button variant="contained" size="large" startIcon={<LoginIcon />} onClick={() => void signIn()} disabled={isLoading}>
          Sign in with Internet Identity
        </Button>
      </CardContent>
    </Card>
  );
};
