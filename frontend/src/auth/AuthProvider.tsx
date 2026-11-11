import { AuthClient } from '@icp-sdk/auth/client';
import type { Identity } from '@icp-sdk/core/agent';
import {
  createContext,
  useCallback,
  useContext,
  useEffect,
  useMemo,
  useState,
  type ReactNode,
} from 'react';
import { iiDerivationOrigin, isLocal } from '../lib/env';

const MAINNET_API = 'https://icp-api.io';

interface AuthState {
  /** Identity for authenticated calls; undefined while signed out or loading. */
  identity?: Identity;
  principal?: string;
  isAuthenticated: boolean;
  isLoading: boolean;
  /** Last sign-in or session error, shown to the user. */
  error?: string;
  clearError: () => void;
  signIn: () => Promise<void>;
  signOut: () => Promise<void>;
}

const AuthContext = createContext<AuthState | null>(null);

// Mainnet Internet Identity (https://id.ai). Local icp-cli networks trust its
// delegations, so the same setup works locally and on mainnet.
//
// The client itself calls II (app_prepare_delegation) on mainnet. On a local
// origin the agent's default host is the local network, which has no II, so
// point those calls at mainnet (with its built-in root key, not the local one).
let client: AuthClient | undefined;
const getAuthClient = () =>
  (client ??= new AuthClient(
    isLocal
      ? { agentOptions: { host: MAINNET_API } }
      : { derivationOrigin: iiDerivationOrigin() },
  ));

export function AuthProvider({ children }: { children: ReactNode }) {
  const [identity, setIdentity] = useState<Identity>();
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<string>();

  const refresh = useCallback(async () => {
    const authClient = getAuthClient();
    try {
      if (authClient.isAuthenticated()) {
        setIdentity(await authClient.getIdentity());
      } else {
        setIdentity(undefined);
      }
    } catch (err) {
      console.warn('Could not restore session', err);
      setError(`Your session could not be restored: ${String((err as Error)?.message ?? err)}`);
      setIdentity(undefined);
    } finally {
      setIsLoading(false);
    }
  }, []);

  useEffect(() => {
    void refresh();
    // Fires on sign-in/out in this or another tab, and on session expiry.
    return getAuthClient().subscribe(() => void refresh());
  }, [refresh]);

  const signIn = useCallback(async () => {
    setIsLoading(true);
    setError(undefined);
    try {
      const id = await getAuthClient().signIn();
      console.info('Signed in as', id.getPrincipal().toText());
      setIdentity(id);
    } catch (err) {
      const msg = String((err as Error)?.message ?? err);
      console.error('Sign-in failed', err);
      // Closing the popup is not an error worth shouting about.
      if (!/clos|cancel|abort/i.test(msg)) setError(`Sign-in failed: ${msg}`);
    } finally {
      setIsLoading(false);
    }
  }, []);

  const signOut = useCallback(async () => {
    await getAuthClient().signOut();
    setIdentity(undefined);
  }, []);

  const value = useMemo<AuthState>(() => {
    const principal = identity?.getPrincipal();
    const authenticated = !!principal && !principal.isAnonymous();
    return {
      identity: authenticated ? identity : undefined,
      principal: authenticated ? principal.toText() : undefined,
      isAuthenticated: authenticated,
      isLoading,
      error,
      clearError: () => setError(undefined),
      signIn,
      signOut,
    };
  }, [identity, isLoading, error, signIn, signOut]);

  return <AuthContext.Provider value={value}>{children}</AuthContext.Provider>;
}

export function useAuth(): AuthState {
  const ctx = useContext(AuthContext);
  if (!ctx) throw new Error('useAuth must be used inside <AuthProvider>');
  return ctx;
}
