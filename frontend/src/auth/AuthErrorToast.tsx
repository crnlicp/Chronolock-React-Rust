import { Alert, Snackbar } from '@mui/material';
import { useAuth } from '../auth/AuthProvider';

/** Shows sign-in and session errors that would otherwise only reach the console. */
export const AuthErrorToast = () => {
  const { error, clearError } = useAuth();
  return (
    <Snackbar
      open={!!error}
      onClose={clearError}
      anchorOrigin={{ vertical: 'bottom', horizontal: 'center' }}
    >
      <Alert severity="error" onClose={clearError} variant="filled">
        {error}
      </Alert>
    </Snackbar>
  );
};
