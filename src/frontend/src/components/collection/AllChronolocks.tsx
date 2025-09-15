import React, { useState, useEffect } from 'react';
import {
  Box,
  Typography,
  CircularProgress,
  Grid,
  Pagination,
} from '@mui/material';
import { useChronolock, Chronolock } from '../../hooks/useChronolock';
import { ChronolockCard } from './ChronolockCard';

export const AllChronolocks: React.FC = () => {
  const {
    getAllChronolocksPaginated,
    getAllChronolocksCount,
    isGetAllChronolocksLoading,
    isGetAllChronolocksCountLoading,
  } = useChronolock();
  const [chronolocks, setChronolocks] = useState<Chronolock[]>([]);
  const [totalCount, setTotalCount] = useState(0);
  const [page, setPage] = useState(1);
  const itemsPerPage = 12;

  // Fetch chronolocks when page changes
  useEffect(() => {
    console.log('AllChronolocks: Fetching chronolocks for page', page);
    const fetchChronolocks = async () => {
      try {
        const offset = (page - 1) * itemsPerPage;
        const result = await getAllChronolocksPaginated(offset, itemsPerPage);
        const chronolocksData = (result as { Ok?: Chronolock[] })?.Ok || [];
        console.log(
          'AllChronolocks: Fetched',
          chronolocksData.length,
          'chronolocks',
        );
        setChronolocks(chronolocksData);
      } catch (error) {
        console.error('Error fetching chronolocks:', error);
        setChronolocks([]); // Set empty array on error
      }
    };

    fetchChronolocks();
  }, [page]); // Only depend on page

  // Fetch count only once on mount
  useEffect(() => {
    console.log('AllChronolocks: Fetching count');
    const fetchCount = async () => {
      try {
        const result = await getAllChronolocksCount();
        const count = result as number;
        console.log('AllChronolocks: Total count is', count);
        setTotalCount(count);
      } catch (error) {
        console.error('Error fetching chronolocks count:', error);
        setTotalCount(0); // Set 0 on error
      }
    };

    fetchCount();
  }, []); // Empty dependency array for mount only

  const totalPages = Math.ceil(Number(totalCount) / itemsPerPage);

  if (isGetAllChronolocksLoading || isGetAllChronolocksCountLoading) {
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
      <Typography variant="h6" gutterBottom>
        Chronolocks ({String(totalCount ?? 0)} total)
      </Typography>

      {chronolocks.length === 0 ? (
        <Box textAlign="center" py={4} my={2} border={'1px solid grey'}>
          <Typography variant="body1" color="white">
            No chronolocks found
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
                sx={{
                  '& .MuiPaginationItem-root': {
                    color: '#fff',
                  },
                }}
              />
            </Box>
          )}
        </>
      )}
    </Box>
  );
};
