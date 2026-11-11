import { Button, Card, CardContent, Typography } from '@mui/material';
import { Link as RouterLink } from 'react-router';

export const NotFoundPage = () => (
  <Card sx={{ maxWidth: 520, mx: 'auto', textAlign: 'center' }}>
    <CardContent sx={{ py: 8 }}>
      <Typography variant="h2" gutterBottom>
        404
      </Typography>
      <Typography color="text.secondary" sx={{ mb: 3 }}>
        This page is locked in another timeline.
      </Typography>
      <Button variant="contained" component={RouterLink} to="/">
        Go home
      </Button>
    </CardContent>
  </Card>
);
