import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  CircularProgress,
  Grid,
  Pagination,
  Chip,
} from '@mui/material';
import { useAuth } from '../../hooks/useAuth';
import { useChronolock, Chronolock } from '../../hooks/useChronolock';
import { ChronolockCard } from './ChronolockCard';

export const DecryptableChronolocks: React.FC = () => {
  const { principal } = useAuth();
  const {
    getUserAccessibleChronolocksPaginated,
    getUserAccessibleChronolocksCount,
    isGetUserAccessibleChronolocksLoading,
    isGetUserAccessibleChronolocksCountLoading,
  } = useChronolock();
  const [chronolocks, setChronolocks] = useState<Chronolock[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [page, setPage] = useState(1);
  const itemsPerPage = 12;

  // Fetch chronolocks when page or principal changes
  useEffect(() => {
    if (!principal) return;

    const fetchChronolocks = async () => {
      try {
        const offset = (page - 1) * itemsPerPage;
        const result = await getUserAccessibleChronolocksPaginated(
          principal,
          offset,
          itemsPerPage,
        );
        const chronolocksData = (result as { Ok?: Chronolock[] })?.Ok || [];

        setChronolocks(chronolocksData);
      } catch (error) {
        console.error('Error fetching decryptable chronolocks:', error);
      }
    };

    fetchChronolocks();
  }, [page, principal]); // Only depend on page and principal

  // Fetch count when principal changes
  useEffect(() => {
    if (!principal) return;

    const fetchCount = async () => {
      try {
        const result = await getUserAccessibleChronolocksCount(principal);
        const count = result as number;
        setTotalCount(count);
      } catch (error) {
        console.error('Error fetching decryptable chronolocks count:', error);
      }
    };

    fetchCount();
  }, [principal]); // Only depend on principal

  const totalPages = Math.ceil(Number(totalCount) / itemsPerPage);

  if (!principal) {
    return (
      <Box textAlign="center" py={4}>
        <Typography variant="body1" color="white">
          Please log in to view chronolocks you can decrypt
        </Typography>
      </Box>
    );
  }

  if (
    isGetUserAccessibleChronolocksLoading ||
    isGetUserAccessibleChronolocksCountLoading
  ) {
    return (
      <Box
        display="flex"
        justifyContent="center"
        alignItems="center"
        minHeight="200px"
      >
        <CircularProgress />
      </Box>
    );
  }

  return (
    <Box>
      <Box display="flex" alignItems="center" gap={2} mb={2}>
        <Typography variant="h6">
          Encrypted Chronolocks for you ({String(totalCount ?? 0)} total)
        </Typography>
        <Chip
          label="Can be opened now"
          color="success"
          size="small"
          variant="outlined"
        />
      </Box>

      {chronolocks.length === 0 ? (
        <Box textAlign="center" py={4}>
          <Typography variant="body1" color="white">
            No chronolocks available for decryption at this time
          </Typography>
          <Typography variant="body2" color="white" mt={1}>
            Check back later or browse other chronolocks
          </Typography>
        </Box>
      ) : (
        <>
          <Grid container spacing={3}>
            {chronolocks.map((chronolock) => (
              <Grid size={{ xs: 12, sm: 6, md: 4, lg: 3 }} key={chronolock.id}>
                <ChronolockCard chronolock={chronolock} />
              </Grid>
            ))}
          </Grid>

          {totalPages > 1 && (
            <Box display="flex" justifyContent="center" mt={4}>
              <Pagination
                count={totalPages}
                page={page}
                onChange={(_, newPage) => setPage(newPage)}
                color="primary"
              />
            </Box>
          )}
        </>
      )}
    </Box>
  );
};
