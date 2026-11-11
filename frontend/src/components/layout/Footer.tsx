import { Box, Container, Link, Stack, Typography } from '@mui/material';
import { Link as RouterLink } from 'react-router';

export const Footer = () => (
  <Box component="footer" sx={{ borderTop: 1, borderColor: 'divider', mt: 10, py: 4 }}>
    <Container maxWidth="lg">
      <Stack
        direction={{ xs: 'column', sm: 'row' }}
        spacing={2}
        sx={{ justifyContent: 'space-between', alignItems: { sm: 'center' } }}
      >
        <Typography variant="body2" color="text.secondary">
          © {new Date().getFullYear()} Chronolock · Built on the Internet Computer
        </Typography>
        <Stack direction="row" spacing={3}>
          {[
            ['/privacy', 'Privacy'],
            ['/terms', 'Terms'],
            ['/cookies', 'Cookies'],
          ].map(([to, label]) => (
            <Link key={to} component={RouterLink} to={to} color="text.secondary" underline="hover" variant="body2">
              {label}
            </Link>
          ))}
          <Link href="https://soonlaunchpad.fun" target="_blank" rel="noopener noreferrer" color="text.secondary" underline="hover" variant="body2">
            $CHRONO on Soon
          </Link>
        </Stack>
      </Stack>
    </Container>
  </Box>
);
