import { createContext, useContext } from 'react';
import { useActorReact } from './hooks/useActorReact';
import { AgentProvider } from '@ic-reactor/react';

const ActorContext = createContext<ReturnType<typeof useActorReact> | null>(
  null,
);

export const ActorContextProvider = ({
  children,
}: {
  children: React.ReactNode;
}) => {
  const actor = useActorReact();
  const host =
    process.env.DFX_NETWORK === 'ic'
      ? 'https://ic0.app'
      : 'http://localhost:4943';
  return (
    <AgentProvider host={host}>
      <ActorContext.Provider value={actor}>
        <actor.CrnlLedgerActorProvider>
          <actor.ChronolockActorProvider>
            {children}
          </actor.ChronolockActorProvider>
        </actor.CrnlLedgerActorProvider>
      </ActorContext.Provider>
    </AgentProvider>
  );
};

export const useActor = () => {
  const ctx = useContext(ActorContext);
  if (!ctx) throw new Error('useActor must be used within ActorProvider');
  return ctx;
};
