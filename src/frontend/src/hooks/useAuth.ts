import { AuthClient } from '@dfinity/auth-client';
import { useEffect, useState } from 'react';
import { useNavigate } from 'react-router';

export const useAuth = () => {
  const navigate = useNavigate();

  const [isAuthenticated, setIsAuthenticated] = useState(false);
  const [principal, setPrincipal] = useState<string | null>(null);
  const [isAuthLoading, setIsAuthLoading] = useState(true);

  async function handleLogin() {
    setIsAuthLoading(true);
    const authClient = await AuthClient.create();
    const isAuthenticated = await authClient.isAuthenticated();
    const provider =
      process.env.DFX_NETWORK === 'ic'
        ? 'https://identity.ic0.app/?#authorize/'
        : `http://${process.env.CANISTER_ID_INTERNET_IDENTITY}.localhost:4943/?#authorize`;

    if (!isAuthenticated) {
      await authClient.login({
        maxTimeToLive: BigInt(7 * 24 * 60 * 60 * 1000 * 1000 * 1000),
        identityProvider: provider,
        onSuccess: () => {
          const identity = authClient.getIdentity();
          setIsAuthenticated(true);
          setPrincipal(identity.getPrincipal().toText());
          setIsAuthLoading(false);
          window.location.reload();
        },
        onError: (error) => {
          console.error('Login failed', error);
          setIsAuthenticated(false);
          setPrincipal(null);
          setIsAuthLoading(false);
        },
      });
    } else {
      console.log('Already authenticated');
    }
  }

  async function handleLogout() {
    setIsAuthLoading(true);
    const authClient = await AuthClient.create();
    await authClient.logout();
    setIsAuthenticated(false);
    setPrincipal(null);
    setIsAuthLoading(false);
    navigate('/');
    window.location.reload();
  }

  useEffect(() => {
    setIsAuthLoading(true);
    // Check authentication status on mount
    AuthClient.create().then((authClient) => {
      authClient.isAuthenticated().then((authenticated) => {
        setIsAuthenticated(authenticated);
        if (authenticated) {
          const identity = authClient.getIdentity();
          setPrincipal(identity.getPrincipal().toText());
        } else {
          setPrincipal(null);
        }
        setIsAuthLoading(false);
      });
    });
  }, []);

  return {
    isAuthenticated,
    principal,
    isAuthLoading,
    handleLogin,
    handleLogout,
  };
};
