import { CssBaseline, ThemeProvider } from '@mui/material';
import { LocalizationProvider } from '@mui/x-date-pickers';
import { AdapterDayjs } from '@mui/x-date-pickers/AdapterDayjs';
import { QueryClient, QueryClientProvider } from '@tanstack/react-query';
import { BrowserRouter, Route, Routes } from 'react-router';
import { AuthProvider } from '../auth/AuthProvider';
import { Layout } from '../components/layout/Layout';
import { CollectionPage } from '../pages/Collection';
import { CreatePage } from '../pages/Create';
import { HomePage } from '../pages/Home';
import { Cookies } from '../pages/legal/Cookies';
import { Privacy } from '../pages/legal/Privacy';
import { Terms } from '../pages/legal/Terms';
import { LockPage } from '../pages/Lock';
import { NotFoundPage } from '../pages/NotFound';
import { WalletPage } from '../pages/Wallet';
import { theme } from './theme';

const queryClient = new QueryClient({
  defaultOptions: { queries: { staleTime: 15_000, retry: 1, refetchOnWindowFocus: false } },
});

export const App = () => (
  <QueryClientProvider client={queryClient}>
    <ThemeProvider theme={theme}>
      <CssBaseline />
      <LocalizationProvider dateAdapter={AdapterDayjs}>
        <AuthProvider>
          <BrowserRouter>
            <Routes>
              <Route element={<Layout />}>
                <Route index element={<HomePage />} />
                <Route path="create" element={<CreatePage />} />
                <Route path="collection" element={<CollectionPage />} />
                <Route path="chronolock/:id" element={<LockPage />} />
                <Route path="wallet" element={<WalletPage />} />
                <Route path="privacy" element={<Privacy />} />
                <Route path="terms" element={<Terms />} />
                <Route path="cookies" element={<Cookies />} />
                <Route path="*" element={<NotFoundPage />} />
              </Route>
            </Routes>
          </BrowserRouter>
        </AuthProvider>
      </LocalizationProvider>
    </ThemeProvider>
  </QueryClientProvider>
);
