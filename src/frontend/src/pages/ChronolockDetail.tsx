import React, { useEffect, useState } from 'react';
import { useNavigate, useParams } from 'react-router';
import { ChronolockCard } from '../components/collection/ChronolockCard';
import { Chronolock, useChronolock } from '../hooks/useChronolock';
import { Box, CircularProgress, Typography } from '@mui/material';

export const ChronolockDetail: React.FC = () => {
  const { id } = useParams<{ id: string }>();
  const [chronolock, setChronolock] = useState<Chronolock>();
  const [error, setError] = useState<string | null>(null);
  const { getChronolock, isGetChronolockLoading } = useChronolock();
  const navigate = useNavigate();

  useEffect(() => {
    if (!id) return;
    setError(null);
    window.scrollTo({ top: 0, behavior: 'smooth' });
    getChronolock(id)
      .then((resp: any) => {
        const chronolockData = (resp as { Ok?: Chronolock }).Ok;
        if (chronolockData) {
          setChronolock(chronolockData);
        } else {
          setError('Chronolock not found!');
        }
      })
      .catch((e: any) => {
        console.error(e);
        setError('Failed to fetch chronolock!');
      });
  }, []);

  function handleNavigate(): void {
    navigate('/');
  }

  if (isGetChronolockLoading) {
    return (
      <div className="container page_container">
        <Box sx={{ display: 'flex', justifyContent: 'center', mt: 4 }}>
          <CircularProgress />
        </Box>
      </div>
    );
  }

  if (error) {
    return (
      <div className="container page_container">
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'center',
            alignItems: 'center',
          }}
          p={3}
        >
          <Typography variant="h6" color="error">
            {error}
          </Typography>
        </Box>
      </div>
    );
  }

  if (!chronolock) {
    return <div className="container page_container"></div>;
  }

  return (
    <div className="container1 page_container1">
      <Box sx={{ maxWidth: 600, margin: '0 auto' }} p={4}>
        <ChronolockCard chronolock={chronolock} onDelete={handleNavigate} />
      </Box>
    </div>
  );
};
