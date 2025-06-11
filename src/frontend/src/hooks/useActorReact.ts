import { createActorContext } from '@ic-reactor/react';
import * as chronolock_candid from '../../../declarations/chronolock_canister';
import * as crnl_ledger_candid from '../../../declarations/crnl_ledger_canister';

type CandidChronolock = typeof chronolock_candid;
type CandidCrnlLedger = typeof crnl_ledger_candid;

export const useActorReact = () => {
  const { ActorProvider: ChronolockActorProvider, ...chronolockActor } =
    createActorContext<CandidChronolock>({
      canisterId: chronolock_candid.canisterId,
      idlFactory: chronolock_candid.idlFactory,
    });

  const { ActorProvider: CrnlLedgerActorProvider, ...crnlLedgerActor } =
    createActorContext<CandidCrnlLedger>({
      canisterId: crnl_ledger_candid.canisterId,
      idlFactory: crnl_ledger_candid.idlFactory,
    });

  return {
    ChronolockActorProvider,
    chronolockActor,
    CrnlLedgerActorProvider,
    crnlLedgerActor,
  };
};
