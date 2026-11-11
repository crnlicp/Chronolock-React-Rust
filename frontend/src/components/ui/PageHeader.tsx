import { Box, Typography } from '@mui/material';
import type { ReactNode } from 'react';

export const PageHeader = ({
  title,
  subtitle,
  action,
}: {
  title: string;
  subtitle?: ReactNode;
  action?: ReactNode;
}) => (
  <Box
    sx={{
      display: 'flex',
      flexWrap: 'wrap',
      alignItems: 'flex-end',
      justifyContent: 'space-between',
      gap: 2,
      mb: 4,
    }}
  >
    <Box sx={{ minWidth: 0 }}>
      <Typography variant="h3" component="h1" sx={{ fontSize: { xs: 30, md: 40 } }}>
        {title}
      </Typography>
      {subtitle && (
        <Typography color="text.secondary" sx={{ mt: 1, maxWidth: 640 }}>
          {subtitle}
        </Typography>
      )}
    </Box>
    {action}
  </Box>
);
