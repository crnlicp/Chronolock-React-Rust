import { Button, Card, CardContent, Typography } from '@mui/material';
import { Component, type ErrorInfo, type ReactNode } from 'react';

interface State {
  error?: Error;
}

/** Keeps a rendering error from blanking the whole app. Resets on navigation via `key`. */
export class ErrorBoundary extends Component<{ children: ReactNode }, State> {
  state: State = {};

  static getDerivedStateFromError(error: Error): State {
    return { error };
  }

  componentDidCatch(error: Error, info: ErrorInfo) {
    console.error('Render error', error, info.componentStack);
  }

  render() {
    if (!this.state.error) return this.props.children;
    return (
      <Card sx={{ maxWidth: 560, mx: 'auto', textAlign: 'center' }}>
        <CardContent sx={{ py: 6 }}>
          <Typography variant="h5" gutterBottom>
            Something went wrong
          </Typography>
          <Typography color="text.secondary" sx={{ mb: 3 }}>
            {this.state.error.message}
          </Typography>
          <Button variant="contained" onClick={() => window.location.reload()}>
            Reload
          </Button>
        </CardContent>
      </Card>
    );
  }
}
