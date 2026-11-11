import { Alert, Box, Card, CardContent, Grid, Pagination, Skeleton, Typography } from '@mui/material';
import { useState } from 'react';
import { type ListKind, useChronolockList } from '../../api/backend';
import { useAuth } from '../../auth/AuthProvider';
import { LockCard } from './LockCard';

const PAGE_SIZE = 12;

export const LockList = ({ kind, empty }: { kind: ListKind; empty: string }) => {
  const { principal } = useAuth();
  const [page, setPage] = useState(1);
  const { data, isLoading, error } = useChronolockList(kind, (page - 1) * PAGE_SIZE, PAGE_SIZE, principal);

  if (error) return <Alert severity="error">Could not load chronolocks.</Alert>;
  if (isLoading || !data) {
    return (
      <Grid container spacing={3}>
        {Array.from({ length: 4 }, (_, i) => (
          <Grid key={i} size={{ xs: 12, sm: 6, md: 4, lg: 3 }}>
            <Skeleton variant="rounded" height={340} />
          </Grid>
        ))}
      </Grid>
    );
  }
  if (data.items.length === 0) {
    return (
      <Card>
        <CardContent sx={{ py: 8, textAlign: 'center' }}>
          <Typography color="text.secondary">{empty}</Typography>
        </CardContent>
      </Card>
    );
  }
  const pages = Math.ceil(Number(data.total) / PAGE_SIZE);
  return (
    <>
      <Grid container spacing={3}>
        {data.items.map((lock) => (
          <Grid key={lock.id.toString()} size={{ xs: 12, sm: 6, md: 4, lg: 3 }}>
            <LockCard lock={lock} />
          </Grid>
        ))}
      </Grid>
      {pages > 1 && (
        <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
          <Pagination count={pages} page={page} onChange={(_, p) => setPage(p)} color="primary" />
        </Box>
      )}
    </>
  );
};
