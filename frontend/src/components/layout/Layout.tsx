import { Box, Container } from '@mui/material';
import { useEffect } from 'react';
import { Outlet, useLocation } from 'react-router';
import { AuthErrorToast } from '../../auth/AuthErrorToast';
import { ErrorBoundary } from './ErrorBoundary';
import { Footer } from './Footer';
import { Header } from './Header';

export const Layout = () => {
  const { pathname } = useLocation();
  useEffect(() => {
    window.scrollTo(0, 0);
  }, [pathname]);
  return (
    <Box sx={{ display: 'flex', flexDirection: 'column', minHeight: '100vh' }}>
      <Header />
      <Container component="main" maxWidth="lg" sx={{ flexGrow: 1, py: { xs: 4, md: 6 } }}>
        <ErrorBoundary key={pathname}>
          <Outlet />
        </ErrorBoundary>
      </Container>
      <Footer />
      <AuthErrorToast />
    </Box>
  );
};
